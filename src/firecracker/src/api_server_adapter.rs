// Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use api_server::{ApiServer, HttpServer, ServerError};
use event_manager::{EventOps, Events, MutEventSubscriber, SubscriberOps};
use logger::{error, warn, ProcessTimeReporter};
use seccompiler::BpfThreadMap;
use utils::epoll::EventSet;
use utils::eventfd::EventFd;
use vmm::resources::VmResources;
use vmm::rpc_interface::{
    ApiRequest, ApiResponse, PrebootApiController, RuntimeApiController, VmmAction,
};
use vmm::vmm_config::instance_info::InstanceInfo;
use vmm::{EventManager, FcExitCode, Vmm};

#[derive(Debug, thiserror::Error)]
pub enum ApiServerError {
    #[error("")]
    MicroVMStoppedWithoutError(FcExitCode),
    #[error("")]
    MicroVMStoppedWithError(FcExitCode),
    #[error("")]
    FailedToBindSocket(String),
    #[error("")]
    FailedToBindAndRunHttpServer(ServerError),
    #[error("")]
    BuildFromJson(crate::BuildFromJsonError),
}

#[derive(Debug)]
struct ApiServerAdapter {
    api_event_fd: EventFd,
    from_api: Receiver<ApiRequest>,
    to_api: Sender<ApiResponse>,
    controller: RuntimeApiController,
}

impl ApiServerAdapter {
    /// Runs the vmm to completion, while any arising control events are deferred
    /// to a `RuntimeApiController`.
    fn run_microvm(
        api_event_fd: EventFd,
        from_api: Receiver<ApiRequest>,
        to_api: Sender<ApiResponse>,
        vm_resources: VmResources,
        vmm: Arc<Mutex<Vmm>>,
        event_manager: &mut EventManager,
    ) -> FcExitCode {
        let api_adapter = Arc::new(Mutex::new(Self {
            api_event_fd,
            from_api,
            to_api,
            controller: RuntimeApiController::new(vm_resources, vmm.clone()),
        }));
        event_manager.add_subscriber(api_adapter);
        loop {
            event_manager
                .run()
                .expect("EventManager events driver fatal error");
            if let Some(exit_code) = vmm.lock().unwrap().shutdown_exit_code() {
                return exit_code;
            }
        }
    }

    fn handle_request(&mut self, req_action: VmmAction) {
        let response = self.controller.handle_request(req_action);
        // Send back the result.
        self.to_api
            .send(Box::new(response))
            .map_err(|_| ())
            .expect("one-shot channel closed");
    }
}
impl MutEventSubscriber for ApiServerAdapter {
    /// Handle a read event (EPOLLIN).
    fn process(&mut self, event: Events, _: &mut EventOps) {
        let source = event.fd();
        let event_set = event.event_set();

        if source == self.api_event_fd.as_raw_fd() && event_set == EventSet::IN {
            let _ = self.api_event_fd.read();
            match self.from_api.try_recv() {
                Ok(api_request) => {
                    let request_is_pause = *api_request == VmmAction::Pause;
                    self.handle_request(*api_request);

                    // If the latest req is a pause request, temporarily switch to a mode where we
                    // do blocking `recv`s on the `from_api` receiver in a loop, until we get
                    // unpaused. The device emulation is implicitly paused since we do not
                    // relinquish control to the event manager because we're not returning from
                    // `process`.
                    if request_is_pause {
                        // This loop only attempts to process API requests, so things like the
                        // metric flush timerfd handling are frozen as well.
                        loop {
                            let req = self.from_api.recv().expect("Error receiving API request.");
                            let req_is_resume = *req == VmmAction::Resume;
                            self.handle_request(*req);
                            if req_is_resume {
                                break;
                            }
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    warn!("");
                }
                Err(TryRecvError::Disconnected) => {
                    panic!("The channel's sending half was disconnected. Cannot receive data.");
                }
            };
        } else {
            error!("");
        }
    }

    fn init(&mut self, ops: &mut EventOps) {
        if let Err(err) = ops.add(Events::new(&self.api_event_fd, EventSet::IN)) {
            error!("Failed to register activate event: {}", err);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_with_api(
    seccomp_filters: &mut BpfThreadMap,
    config_json: Option<String>,
    bind_path: PathBuf,
    instance_info: InstanceInfo,
    process_time_reporter: ProcessTimeReporter,
    boot_timer_enabled: bool,
    api_payload_limit: usize,
    mmds_size_limit: usize,
    metadata_json: Option<&str>,
) -> Result<(), ApiServerError> {
    // FD to notify of API events. This is a blocking eventfd by design.
    // It is used in the config/pre-boot loop which is a simple blocking loop
    // which only consumes API events.
    let api_event_fd = EventFd::new(libc::EFD_SEMAPHORE).expect("Cannot create API Eventfd.");

    // Channels for both directions between Vmm and Api threads.
    let (to_vmm, from_api) = channel();
    let (to_api, from_vmm) = channel();

    let to_vmm_event_fd = api_event_fd
        .try_clone()
        .expect("Failed to clone API event FD");
    let api_seccomp_filter = seccomp_filters
        .remove("api")
        .expect("Missing seccomp filter for API thread.");

    let server = match HttpServer::new(&bind_path) {
        Ok(s) => s,
        Err(ServerError::IOError(inner)) if inner.kind() == std::io::ErrorKind::AddrInUse => {
            let sock_path = bind_path.display().to_string();
            return Err(ApiServerError::FailedToBindSocket(sock_path));
        }
        Err(err) => {
            return Err(ApiServerError::FailedToBindAndRunHttpServer(err));
        }
    };

    // Start the separate API thread.
    let api_thread = thread::Builder::new()
        .name("fc_api".to_owned())
        .spawn(move || {
            ApiServer::new(to_vmm, from_vmm, to_vmm_event_fd).run(
                server,
                process_time_reporter,
                &api_seccomp_filter,
                api_payload_limit,
            );
        })
        .expect("API thread spawn failed.");

    let mut event_manager = EventManager::new().expect("Unable to create EventManager");

    // Create the firecracker metrics object responsible for periodically printing metrics.
    let firecracker_metrics = Arc::new(Mutex::new(super::metrics::PeriodicMetrics::new()));
    event_manager.add_subscriber(firecracker_metrics.clone());

    // Configure, build and start the microVM.
    let build_result = match config_json {
        Some(json) => super::build_microvm_from_json(
            seccomp_filters,
            &mut event_manager,
            json,
            instance_info,
            boot_timer_enabled,
            mmds_size_limit,
            metadata_json,
        )
        .map_err(ApiServerError::BuildFromJson),
        None => PrebootApiController::build_microvm_from_requests(
            seccomp_filters,
            &mut event_manager,
            instance_info,
            &from_api,
            &to_api,
            &api_event_fd,
            boot_timer_enabled,
            mmds_size_limit,
            metadata_json,
        )
        .map_err(ApiServerError::MicroVMStoppedWithError),
    };

    let result = build_result.map(|(vm_resources, vmm)| {
        firecracker_metrics
            .lock()
            .expect("Poisoned lock")
            .start(super::metrics::WRITE_METRICS_PERIOD_MS);

        ApiServerAdapter::run_microvm(
            api_event_fd,
            from_api,
            to_api,
            vm_resources,
            vmm,
            &mut event_manager,
        )
    });

    // We want to tell the API thread to shut down for a clean exit. But this is after
    // the Vmm.stop() has been called, so it's a moment of internal finalization (as
    // opposed to be something the client might call to shut the Vm down).  Since it's
    // an internal signal implementing it with an HTTP request is probably not the ideal
    // way to do it...but having another way would involve multiplexing micro-http server
    // with some other communication mechanism, or enhancing micro-http with exit
    // conditions.

    // We also need to make sure the socket path is ready.
    let mut sock = UnixStream::connect(bind_path).unwrap();
    sock.write_all(b"PUT /shutdown-internal HTTP/1.1\r\n\r\n")
        .unwrap();

    // This call to thread::join() should block until the API thread has processed the
    // shutdown-internal and returns from its function.
    api_thread.join().expect("Api thread should join");

    match result {
        Ok(exit_code) => Err(ApiServerError::MicroVMStoppedWithoutError(exit_code)),
        Err(exit_error) => Err(exit_error),
    }
}
