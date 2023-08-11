// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

//! Virtual Machine Monitor that leverages the Linux Kernel-based Virtual Machine (KVM),
//! and other virtualization features to run a single lightweight micro-virtual
//! machine (microVM).
#![deny(missing_docs)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::blanket_clippy_restriction_lints)]

/// Architecture specific bindings.
#[allow(missing_docs)]
pub mod arch_gen;

/// Implements platform specific functionality.
/// Supported platforms: x86_64 and aarch64.
pub mod arch;

/// High-level interface over Linux io_uring.
///
/// Aims to provide an easy-to-use interface, while making some Firecracker-specific simplifying
/// assumptions. The crate does not currently aim at supporting all io_uring features and use
/// cases. For example, it only works with pre-registered fds and read/write/fsync requests.
///
/// Requires at least kernel version 5.10.51.
/// For more information on io_uring, refer to the man pages.
/// [This pdf](https://kernel.dk/io_uring.pdf) is also very useful, though outdated at times.
pub mod io_uring;

/// # Rate Limiter
///
/// Provides a rate limiter written in Rust useful for IO operations that need to
/// be throttled.
///
/// ## Behavior
///
/// The rate limiter starts off as 'unblocked' with two token buckets configured
/// with the values passed in the `RateLimiter::new()` constructor.
/// All subsequent accounting is done independently for each token bucket based
/// on the `TokenType` used. If any of the buckets runs out of budget, the limiter
/// goes in the 'blocked' state. At this point an internal timer is set up which
/// will later 'wake up' the user in order to retry sending data. The 'wake up'
/// notification will be dispatched as an event on the FD provided by the `AsRawFD`
/// trait implementation.
///
/// The contract is that the user shall also call the `event_handler()` method on
/// receipt of such an event.
///
/// The token buckets are replenished when a called `consume()` doesn't find enough
/// tokens in the bucket. The amount of tokens replenished is automatically calculated
/// to respect the `complete_refill_time` configuration parameter provided by the user.
/// The token buckets will never replenish above their respective `size`.
///
/// Each token bucket can start off with a `one_time_burst` initial extra capacity
/// on top of their `size`. This initial extra credit does not replenish and
/// can be used for an initial burst of data.
///
/// The granularity for 'wake up' events when the rate limiter is blocked is
/// currently hardcoded to `100 milliseconds`.
///
/// ## Limitations
///
/// This rate limiter implementation relies on the *Linux kernel's timerfd* so its
/// usage is limited to Linux systems.
///
/// Another particularity of this implementation is that it is not self-driving.
/// It is meant to be used in an external event loop and thus implements the `AsRawFd`
/// trait and provides an *event-handler* as part of its API. This *event-handler*
/// needs to be called by the user on every event on the rate limiter's `AsRawFd` FD.
pub mod rate_limiter;

/// Handles setup and initialization a `Vmm` object.
pub mod builder;
/// Types for guest configuration.
pub mod cpu_config;
pub(crate) mod device_manager;
/// Emulates virtual and hardware devices.
#[allow(missing_docs)]
pub mod devices;
pub mod memory_snapshot;
/// Save/restore utilities.
pub mod persist;
/// Resource store for configured microVM resources.
pub mod resources;
/// microVM RPC API adapters.
pub mod rpc_interface;
/// Seccomp filter utilities.
pub mod seccomp_filters;
/// Signal handling utilities.
pub mod signal_handler;
/// Utility functions for integration and benchmark testing
pub mod utilities;
/// microVM state versions.
pub mod version_map;
/// Wrappers over structures used to configure the VMM.
pub mod vmm_config;

mod vstate;

use std::collections::HashMap;
use std::io;
use std::os::unix::io::AsRawFd;
use std::sync::mpsc::{RecvTimeoutError, TryRecvError};
use std::sync::{Arc, Barrier, Mutex};
use std::time::Duration;

use event_manager::{EventManager as BaseEventManager, EventOps, Events, MutEventSubscriber};
use logger::{error, info, warn, MetricsError, METRICS};
use seccompiler::BpfProgram;
use snapshot::Persist;
use userfaultfd::Uffd;
use utils::epoll::EventSet;
use utils::eventfd::EventFd;
use utils::terminal::Terminal;
use utils::vm_memory::{GuestMemory, GuestMemoryMmap, GuestMemoryRegion};
use vstate::vcpu::{self, KvmVcpuConfigureError, StartThreadedError, VcpuSendEventError};

use crate::arch::DeviceType;
use crate::cpu_config::templates::CpuConfiguration;
#[cfg(target_arch = "x86_64")]
use crate::device_manager::legacy::PortIODeviceManager;
use crate::device_manager::mmio::MMIODeviceManager;
use crate::devices::legacy::{IER_RDA_BIT, IER_RDA_OFFSET};
use crate::devices::virtio::balloon::BalloonError;
use crate::devices::virtio::{
    Balloon, BalloonConfig, BalloonStats, Block, Net, BALLOON_DEV_ID, TYPE_BALLOON, TYPE_BLOCK,
    TYPE_NET,
};
use crate::memory_snapshot::SnapshotMemory;
use crate::persist::{MicrovmState, MicrovmStateError, VmInfo};
use crate::rate_limiter::BucketUpdate;
use crate::vmm_config::instance_info::{InstanceInfo, VmState};
use crate::vstate::vcpu::VcpuState;
pub use crate::vstate::vcpu::{Vcpu, VcpuConfig, VcpuEvent, VcpuHandle, VcpuResponse};
pub use crate::vstate::vm::Vm;

/// Shorthand type for the EventManager flavour used by Firecracker.
pub type EventManager = BaseEventManager<Arc<Mutex<dyn MutEventSubscriber>>>;

// Since the exit code names e.g. `SIGBUS` are most appropriate yet trigger a test error with the
// clippy lint `upper_case_acronyms` we have disabled this lint for this enum.
/// Vmm exit-code type.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FcExitCode {
    /// Success exit code.
    Ok = 0,
    /// Generic error exit code.
    GenericError = 1,
    /// Generic exit code for an error considered not possible to occur if the program logic is
    /// sound.
    UnexpectedError = 2,
    /// Firecracker was shut down after intercepting a restricted system call.
    BadSyscall = 148,
    /// Firecracker was shut down after intercepting `SIGBUS`.
    SIGBUS = 149,
    /// Firecracker was shut down after intercepting `SIGSEGV`.
    SIGSEGV = 150,
    /// Firecracker was shut down after intercepting `SIGXFSZ`.
    SIGXFSZ = 151,
    /// Firecracker was shut down after intercepting `SIGXCPU`.
    SIGXCPU = 154,
    /// Firecracker was shut down after intercepting `SIGPIPE`.
    SIGPIPE = 155,
    /// Firecracker was shut down after intercepting `SIGHUP`.
    SIGHUP = 156,
    /// Firecracker was shut down after intercepting `SIGILL`.
    SIGILL = 157,
    /// Bad configuration for microvm's resources, when using a single json.
    BadConfiguration = 152,
    /// Command line arguments parsing error.
    ArgParsing = 153,
}

/// Timeout used in recv_timeout, when waiting for a vcpu response on
/// Pause/Resume/Save/Restore. A high enough limit that should not be reached during normal usage,
/// used to detect a potential vcpu deadlock.
pub const RECV_TIMEOUT_SEC: Duration = Duration::from_secs(30);

/// Default byte limit of accepted http requests on API and MMDS servers.
pub const HTTP_MAX_PAYLOAD_SIZE: usize = 51200;

/// Errors associated with the VMM internal logic. These errors cannot be generated by direct user
/// input, but can result from bad configuration of the host (for example if Firecracker doesn't
/// have permissions to open the KVM fd).
#[derive(Debug, thiserror::Error)]
pub enum VmmError {
    #[cfg(target_arch = "aarch64")]
    #[error("Invalid cmdline")]
    /// Invalid command line error.
    Cmdline,
    /// Device manager error.
    #[error("{0}")]
    DeviceManager(device_manager::mmio::MmioError),
    /// Cannot fetch the KVM dirty bitmap.
    #[error("Error getting the KVM dirty bitmap. {0}")]
    DirtyBitmap(kvm_ioctls::Error),
    /// Cannot read from an Event file descriptor.
    #[error("Event fd error: {0}")]
    EventFd(io::Error),
    /// I8042 Error.
    #[error("I8042 error: {0}")]
    I8042Error(devices::legacy::I8042DeviceError),
    /// Cannot access kernel file.
    #[error("Cannot access kernel file: {0}")]
    KernelFile(io::Error),
    /// Cannot open /dev/kvm. Either the host does not have KVM or Firecracker does not have
    /// permission to open the file descriptor.
    #[error("Failed to validate KVM support: {0}")]
    KvmContext(vstate::system::SystemError),
    /// Cannot add devices to the Legacy I/O Bus.
    #[cfg(target_arch = "x86_64")]
    #[error("Cannot add devices to the legacy I/O Bus. {0}")]
    LegacyIOBus(device_manager::legacy::LegacyDeviceError),
    /// Internal metrics system error.
    #[error("Metrics error: {0}")]
    Metrics(MetricsError),
    /// Cannot add a device to the MMIO Bus.
    #[error("Cannot add a device to the MMIO Bus. {0}")]
    RegisterMMIODevice(device_manager::mmio::MmioError),
    /// Cannot install seccomp filters.
    #[error("Cannot install seccomp filters: {0}")]
    SeccompFilters(seccompiler::InstallationError),
    /// Write to the serial console failed.
    #[error("Error writing to the serial console: {0}")]
    Serial(io::Error),
    /// Cannot create Timer file descriptor.
    #[error("Error creating timer fd: {0}")]
    TimerFd(io::Error),
    /// Vcpu configuration error.
    #[error("Error configuring the vcpu for boot: {0}")]
    VcpuConfigure(KvmVcpuConfigureError),
    /// Vcpu create error.
    #[error("Error creating the vcpu: {0}")]
    VcpuCreate(vstate::vcpu::VcpuError),
    /// Cannot send event to vCPU.
    #[error("Cannot send event to vCPU. {0}")]
    VcpuEvent(vstate::vcpu::VcpuError),
    /// Cannot create a vCPU handle.
    #[error("Cannot create a vCPU handle. {0}")]
    VcpuHandle(vstate::vcpu::VcpuError),
    /// Vcpu init error.
    #[cfg(target_arch = "aarch64")]
    #[error("Error initializing the vcpu: {0}")]
    VcpuInit(vstate::vcpu::KvmVcpuError),
    /// vCPU start error.
    #[error("Failed to start vCPUs")]
    VcpuStart(StartVcpusError),
    /// vCPU pause failed.
    #[error("Failed to pause the vCPUs.")]
    VcpuPause,
    /// vCPU exit failed.
    #[error("Failed to exit the vCPUs.")]
    VcpuExit,
    /// vCPU resume failed.
    #[error("Failed to resume the vCPUs.")]
    VcpuResume,
    /// Vcpu send message failed.
    #[error("Failed to message the vCPUs.")]
    VcpuMessage,
    /// Cannot spawn a new Vcpu thread.
    #[error("Cannot spawn Vcpu thread: {0}")]
    VcpuSpawn(io::Error),
    /// Vm error.
    #[error("Vm error: {0}")]
    Vm(vstate::vm::VmError),
    /// Error thrown by observer object on Vmm initialization.
    #[error("Error thrown by observer object on Vmm initialization: {0}")]
    VmmObserverInit(utils::errno::Error),
    /// Error thrown by observer object on Vmm teardown.
    #[error("Error thrown by observer object on Vmm teardown: {0}")]
    VmmObserverTeardown(utils::errno::Error),
}

/// Shorthand type for KVM dirty page bitmap.
pub type DirtyBitmap = HashMap<usize, Vec<u64>>;

#[tracing::instrument(level = "trace", skip(guest_memory))]
/// Returns the size of guest memory, in MiB.
pub(crate) fn mem_size_mib(guest_memory: &GuestMemoryMmap) -> u64 {
    guest_memory.iter().map(|region| region.len()).sum::<u64>() >> 20
}

/// Error type for [`Vmm::emulate_serial_init`].
#[derive(Debug, derive_more::From, thiserror::Error)]
#[error("Emulate serial init error: {0}")]
pub struct EmulateSerialInitError(std::io::Error);

/// Error type for [`Vmm::start_vcpus`].
#[derive(Debug, thiserror::Error)]
pub enum StartVcpusError {
    /// Vmm observer init error.
    #[error("{0}")]
    VmmObserverInit(#[from] utils::errno::Error),
    /// vCPU handle error.
    #[error("{0}")]
    VcpuHandle(#[from] StartThreadedError),
}

/// Error type for [`Vmm::restore_vcpu_states`]
#[derive(Debug, thiserror::Error)]
pub enum RestoreVcpusError {
    /// Invalid input.
    #[error("Invalid input.")]
    InvalidInput,
    /// Failed to send event.
    #[error("Failed to send event: {0}")]
    SendEvent(#[from] VcpuSendEventError),
    /// Vcpu is in unexpected state.
    #[error("Unexpected vCPU response.")]
    UnexpectedVcpuResponse,
    /// Failed to restore vCPU state.
    #[error("Failed to restore vCPU state: {0}")]
    RestoreVcpuState(#[from] vcpu::VcpuError),
    /// Not allowed.
    #[error("Not allowed: {0}")]
    NotAllowed(String),
}

/// Error type for [`Vmm::dump_cpu_config()`]
#[derive(Debug, thiserror::Error)]
pub enum DumpCpuConfigError {
    /// Failed to send an event to vcpu thread.
    #[error("Failed to send event to vcpu thread: {0:?}")]
    SendEvent(#[from] VcpuSendEventError),
    /// Got an unexpected response from vcpu thread.
    #[error("Got unexpected response from vcpu thread.")]
    UnexpectedResponse,
    /// Failed to dump CPU config.
    #[error("Failed to dump CPU config: {0}")]
    DumpCpuConfig(#[from] vcpu::VcpuError),
    /// Operation not allowed.
    #[error("Operation not allowed: {0}")]
    NotAllowed(String),
}

/// Contains the state and associated methods required for the Firecracker VMM.
#[derive(Debug)]
pub struct Vmm {
    events_observer: Option<std::io::Stdin>,
    instance_info: InstanceInfo,
    shutdown_exit_code: Option<FcExitCode>,

    // Guest VM core resources.
    vm: Vm,
    guest_memory: GuestMemoryMmap,
    // Save UFFD in order to keep it open in the Firecracker process, as well.
    // Since this field is never read again, we need to allow `dead_code`.
    #[allow(dead_code)]
    uffd: Option<Uffd>,
    vcpus_handles: Vec<VcpuHandle>,
    // Used by Vcpus and devices to initiate teardown; Vmm should never write here.
    vcpus_exit_evt: EventFd,

    // Guest VM devices.
    mmio_device_manager: MMIODeviceManager,
    #[cfg(target_arch = "x86_64")]
    pio_device_manager: PortIODeviceManager,
}

impl Vmm {
    #[tracing::instrument(level = "trace", skip(self))]
    /// Gets Vmm version.
    pub fn version(&self) -> String {
        self.instance_info.vmm_version.clone()
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Gets Vmm instance info.
    pub fn instance_info(&self) -> InstanceInfo {
        self.instance_info.clone()
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Provides the Vmm shutdown exit code if there is one.
    pub fn shutdown_exit_code(&self) -> Option<FcExitCode> {
        self.shutdown_exit_code
    }

    #[tracing::instrument(level = "trace", skip(self,device_type,device_id))]
    /// Gets the specified bus device.
    pub fn get_bus_device(
        &self,
        device_type: DeviceType,
        device_id: &str,
    ) -> Option<&Mutex<devices::bus::BusDevice>> {
        self.mmio_device_manager.get_device(device_type, device_id)
    }

    #[tracing::instrument(level = "trace", skip(self,vcpus,vcpu_seccomp_filter))]
    /// Starts the microVM vcpus.
    ///
    /// # Errors
    ///
    /// When:
    /// - [`vmm::VmmEventsObserver::on_vmm_boot`] errors.
    /// - [`vmm::vstate::vcpu::Vcpu::start_threaded`] errors.
    pub fn start_vcpus(
        &mut self,
        mut vcpus: Vec<Vcpu>,
        vcpu_seccomp_filter: Arc<BpfProgram>,
    ) -> Result<(), StartVcpusError> {
        let vcpu_count = vcpus.len();
        let barrier = Arc::new(Barrier::new(vcpu_count + 1));

        if let Some(stdin) = self.events_observer.as_mut() {
            // Set raw mode for stdin.
            stdin.lock().set_raw_mode().map_err(|err| {
                warn!("Cannot set raw mode for the terminal. {:?}", err);
                err
            })?;

            // Set non blocking stdin.
            stdin.lock().set_non_block(true).map_err(|err| {
                warn!("Cannot set non block for the terminal. {:?}", err);
                err
            })?;
        }

        Vcpu::register_kick_signal_handler();

        self.vcpus_handles.reserve(vcpu_count);

        for mut vcpu in vcpus.drain(..) {
            vcpu.set_mmio_bus(self.mmio_device_manager.bus.clone());
            #[cfg(target_arch = "x86_64")]
            vcpu.kvm_vcpu
                .set_pio_bus(self.pio_device_manager.io_bus.clone());

            self.vcpus_handles
                .push(vcpu.start_threaded(vcpu_seccomp_filter.clone(), barrier.clone())?);
        }
        self.instance_info.state = VmState::Paused;
        // Wait for vCPUs to initialize their TLS before moving forward.
        barrier.wait();

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Sends a resume command to the vCPUs.
    pub fn resume_vm(&mut self) -> Result<(), VmmError> {
        self.mmio_device_manager.kick_devices();

        // Send the events.
        self.vcpus_handles
            .iter()
            .try_for_each(|handle| handle.send_event(VcpuEvent::Resume))
            .map_err(|_| VmmError::VcpuMessage)?;

        // Check the responses.
        if self
            .vcpus_handles
            .iter()
            .map(|handle| handle.response_receiver().recv_timeout(RECV_TIMEOUT_SEC))
            .any(|response| !matches!(response, Ok(VcpuResponse::Resumed)))
        {
            return Err(VmmError::VcpuMessage);
        }

        self.instance_info.state = VmState::Running;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Sends a pause command to the vCPUs.
    pub fn pause_vm(&mut self) -> Result<(), VmmError> {
        // Send the events.
        self.vcpus_handles
            .iter()
            .try_for_each(|handle| handle.send_event(VcpuEvent::Pause))
            .map_err(|_| VmmError::VcpuMessage)?;

        // Check the responses.
        if self
            .vcpus_handles
            .iter()
            .map(|handle| handle.response_receiver().recv_timeout(RECV_TIMEOUT_SEC))
            .any(|response| !matches!(response, Ok(VcpuResponse::Paused)))
        {
            return Err(VmmError::VcpuMessage);
        }

        self.instance_info.state = VmState::Paused;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Returns a reference to the inner `GuestMemoryMmap` object.
    pub fn guest_memory(&self) -> &GuestMemoryMmap {
        &self.guest_memory
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Sets RDA bit in serial console
    pub fn emulate_serial_init(&self) -> Result<(), EmulateSerialInitError> {
        // When restoring from a previously saved state, there is no serial
        // driver initialization, therefore the RDA (Received Data Available)
        // interrupt is not enabled. Because of that, the driver won't get
        // notified of any bytes that we send to the guest. The clean solution
        // would be to save the whole serial device state when we do the vm
        // serialization. For now we set that bit manually

        #[cfg(target_arch = "aarch64")]
        {
            let serial_bus_device = self.get_bus_device(DeviceType::Serial, "Serial");
            if serial_bus_device.is_none() {
                return Ok(());
            }
            let mut serial_device_locked =
                serial_bus_device.unwrap().lock().expect("Poisoned lock");
            let serial = serial_device_locked
                .serial_mut()
                .expect("Unexpected BusDeviceType");

            serial
                .serial
                .write(IER_RDA_OFFSET, IER_RDA_BIT)
                .map_err(|_| EmulateSerialInitError(std::io::Error::last_os_error()))?;
            Ok(())
        }

        #[cfg(target_arch = "x86_64")]
        {
            let mut guard = self
                .pio_device_manager
                .stdio_serial
                .lock()
                .expect("Poisoned lock");
            let serial = guard.serial_mut().unwrap();

            serial
                .serial
                .write(IER_RDA_OFFSET, IER_RDA_BIT)
                .map_err(|_| EmulateSerialInitError(std::io::Error::last_os_error()))?;
            Ok(())
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Injects CTRL+ALT+DEL keystroke combo in the i8042 device.
    #[cfg(target_arch = "x86_64")]
    pub fn send_ctrl_alt_del(&mut self) -> Result<(), VmmError> {
        self.pio_device_manager
            .i8042
            .lock()
            .expect("i8042 lock was poisoned")
            .i8042_device_mut()
            .unwrap()
            .trigger_ctrl_alt_del()
            .map_err(VmmError::I8042Error)
    }

    #[tracing::instrument(level = "trace", skip(self,vm_info))]
    /// Saves the state of a paused Microvm.
    pub fn save_state(&mut self, vm_info: &VmInfo) -> Result<MicrovmState, MicrovmStateError> {
        use self::MicrovmStateError::SaveVmState;
        let vcpu_states = self.save_vcpu_states()?;
        let vm_state = {
            #[cfg(target_arch = "x86_64")]
            {
                self.vm.save_state().map_err(SaveVmState)?
            }
            #[cfg(target_arch = "aarch64")]
            {
                let mpidrs = construct_kvm_mpidrs(&vcpu_states);

                self.vm.save_state(&mpidrs).map_err(SaveVmState)?
            }
        };
        let device_states = self.mmio_device_manager.save();

        let memory_state = self.guest_memory().describe();

        Ok(MicrovmState {
            vm_info: vm_info.clone(),
            memory_state,
            vm_state,
            vcpu_states,
            device_states,
        })
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn save_vcpu_states(&mut self) -> Result<Vec<VcpuState>, MicrovmStateError> {
        for handle in self.vcpus_handles.iter() {
            handle
                .send_event(VcpuEvent::SaveState)
                .map_err(MicrovmStateError::SignalVcpu)?;
        }

        let vcpu_responses = self
            .vcpus_handles
            .iter()
            // `Iterator::collect` can transform a `Vec<Result>` into a `Result<Vec>`.
            .map(|handle| handle.response_receiver().recv_timeout(RECV_TIMEOUT_SEC))
            .collect::<Result<Vec<VcpuResponse>, RecvTimeoutError>>()
            .map_err(|_| MicrovmStateError::UnexpectedVcpuResponse)?;

        let vcpu_states = vcpu_responses
            .into_iter()
            .map(|response| match response {
                VcpuResponse::SavedState(state) => Ok(*state),
                VcpuResponse::Error(err) => Err(MicrovmStateError::SaveVcpuState(err)),
                VcpuResponse::NotAllowed(reason) => Err(MicrovmStateError::NotAllowed(reason)),
                _ => Err(MicrovmStateError::UnexpectedVcpuResponse),
            })
            .collect::<Result<Vec<VcpuState>, MicrovmStateError>>()?;

        Ok(vcpu_states)
    }

    #[tracing::instrument(level = "trace", skip(self,vcpu_states))]
    /// Restores vcpus kvm states.
    pub fn restore_vcpu_states(
        &mut self,
        mut vcpu_states: Vec<VcpuState>,
    ) -> Result<(), RestoreVcpusError> {
        if vcpu_states.len() != self.vcpus_handles.len() {
            return Err(RestoreVcpusError::InvalidInput);
        }
        for (handle, state) in self.vcpus_handles.iter().zip(vcpu_states.drain(..)) {
            handle.send_event(VcpuEvent::RestoreState(Box::new(state)))?;
        }

        let vcpu_responses = self
            .vcpus_handles
            .iter()
            // `Iterator::collect` can transform a `Vec<Result>` into a `Result<Vec>`.
            .map(|handle| handle.response_receiver().recv_timeout(RECV_TIMEOUT_SEC))
            .collect::<Result<Vec<VcpuResponse>, RecvTimeoutError>>()
            .map_err(|_| RestoreVcpusError::UnexpectedVcpuResponse)?;

        for response in vcpu_responses.into_iter() {
            match response {
                VcpuResponse::RestoredState => Ok(()),
                VcpuResponse::Error(err) => Err(RestoreVcpusError::RestoreVcpuState(err)),
                VcpuResponse::NotAllowed(reason) => Err(RestoreVcpusError::NotAllowed(reason)),
                _ => Err(RestoreVcpusError::UnexpectedVcpuResponse),
            }?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Dumps CPU configuration.
    pub fn dump_cpu_config(&mut self) -> Result<Vec<CpuConfiguration>, DumpCpuConfigError> {
        for handle in self.vcpus_handles.iter() {
            handle
                .send_event(VcpuEvent::DumpCpuConfig)
                .map_err(DumpCpuConfigError::SendEvent)?;
        }

        let vcpu_responses = self
            .vcpus_handles
            .iter()
            .map(|handle| handle.response_receiver().recv_timeout(RECV_TIMEOUT_SEC))
            .collect::<Result<Vec<VcpuResponse>, RecvTimeoutError>>()
            .map_err(|_| DumpCpuConfigError::UnexpectedResponse)?;

        let cpu_configs = vcpu_responses
            .into_iter()
            .map(|response| match response {
                VcpuResponse::DumpedCpuConfig(cpu_config) => Ok(*cpu_config),
                VcpuResponse::Error(err) => Err(DumpCpuConfigError::DumpCpuConfig(err)),
                VcpuResponse::NotAllowed(reason) => Err(DumpCpuConfigError::NotAllowed(reason)),
                _ => Err(DumpCpuConfigError::UnexpectedResponse),
            })
            .collect::<Result<Vec<CpuConfiguration>, DumpCpuConfigError>>()?;

        Ok(cpu_configs)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Retrieves the KVM dirty bitmap for each of the guest's memory regions.
    pub fn get_dirty_bitmap(&self) -> Result<DirtyBitmap, VmmError> {
        let mut bitmap: DirtyBitmap = HashMap::new();
        self.guest_memory
            .iter()
            .enumerate()
            .try_for_each(|(slot, region)| {
                let bitmap_region = self
                    .vm
                    .fd()
                    .get_dirty_log(slot as u32, region.len() as usize)?;
                bitmap.insert(slot, bitmap_region);
                Ok(())
            })
            .map_err(VmmError::DirtyBitmap)?;
        Ok(bitmap)
    }

    #[tracing::instrument(level = "trace", skip(self,enable))]
    /// Enables or disables KVM dirty page tracking.
    pub fn set_dirty_page_tracking(&mut self, enable: bool) -> Result<(), VmmError> {
        // This function _always_ results in an ioctl update. The VMM is stateless in the sense
        // that it's unaware of the current dirty page tracking setting.
        // The VMM's consumer will need to cache the dirty tracking setting internally. For
        // example, if this function were to be exposed through the VMM controller, the VMM
        // resources should cache the flag.
        self.vm
            .set_kvm_memory_regions(&self.guest_memory, enable)
            .map_err(VmmError::Vm)
    }

    #[tracing::instrument(level = "trace", skip(self,drive_id,path_on_host))]
    /// Updates the path of the host file backing the emulated block device with id `drive_id`.
    /// We update the disk image on the device and its virtio configuration.
    pub fn update_block_device_path(
        &mut self,
        drive_id: &str,
        path_on_host: String,
    ) -> Result<(), VmmError> {
        self.mmio_device_manager
            .with_virtio_device_with_id(TYPE_BLOCK, drive_id, |block: &mut Block| {
                block
                    .update_disk_image(path_on_host)
                    .map_err(|err| format!("{:?}", err))
            })
            .map_err(VmmError::DeviceManager)
    }

    #[tracing::instrument(level = "trace", skip(self,drive_id,rl_bytes,rl_ops))]
    /// Updates the rate limiter parameters for block device with `drive_id` id.
    pub fn update_block_rate_limiter(
        &mut self,
        drive_id: &str,
        rl_bytes: BucketUpdate,
        rl_ops: BucketUpdate,
    ) -> Result<(), VmmError> {
        self.mmio_device_manager
            .with_virtio_device_with_id(TYPE_BLOCK, drive_id, |block: &mut Block| {
                block.update_rate_limiter(rl_bytes, rl_ops);
                Ok(())
            })
            .map_err(VmmError::DeviceManager)
    }

    #[tracing::instrument(level = "trace", skip(self,net_id,rx_bytes,rx_ops,tx_bytes,tx_ops))]
    /// Updates the rate limiter parameters for net device with `net_id` id.
    pub fn update_net_rate_limiters(
        &mut self,
        net_id: &str,
        rx_bytes: BucketUpdate,
        rx_ops: BucketUpdate,
        tx_bytes: BucketUpdate,
        tx_ops: BucketUpdate,
    ) -> Result<(), VmmError> {
        self.mmio_device_manager
            .with_virtio_device_with_id(TYPE_NET, net_id, |net: &mut Net| {
                net.patch_rate_limiters(rx_bytes, rx_ops, tx_bytes, tx_ops);
                Ok(())
            })
            .map_err(VmmError::DeviceManager)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Returns a reference to the balloon device if present.
    pub fn balloon_config(&self) -> Result<BalloonConfig, BalloonError> {
        if let Some(busdev) = self.get_bus_device(DeviceType::Virtio(TYPE_BALLOON), BALLOON_DEV_ID)
        {
            let virtio_device = busdev
                .lock()
                .expect("Poisoned lock")
                .mmio_transport_ref()
                .expect("Unexpected device type")
                .device();

            let config = virtio_device
                .lock()
                .expect("Poisoned lock")
                .as_mut_any()
                .downcast_mut::<Balloon>()
                .unwrap()
                .config();

            Ok(config)
        } else {
            Err(BalloonError::DeviceNotFound)
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Returns the latest balloon statistics if they are enabled.
    pub fn latest_balloon_stats(&self) -> Result<BalloonStats, BalloonError> {
        if let Some(busdev) = self.get_bus_device(DeviceType::Virtio(TYPE_BALLOON), BALLOON_DEV_ID)
        {
            let virtio_device = busdev
                .lock()
                .expect("Poisoned lock")
                .mmio_transport_ref()
                .expect("Unexpected device type")
                .device();

            let latest_stats = virtio_device
                .lock()
                .expect("Poisoned lock")
                .as_mut_any()
                .downcast_mut::<Balloon>()
                .unwrap()
                .latest_stats()
                .ok_or(BalloonError::StatisticsDisabled)
                .map(|stats| stats.clone())?;

            Ok(latest_stats)
        } else {
            Err(BalloonError::DeviceNotFound)
        }
    }

    #[tracing::instrument(level = "trace", skip(self,amount_mib))]
    /// Updates configuration for the balloon device target size.
    pub fn update_balloon_config(&mut self, amount_mib: u32) -> Result<(), BalloonError> {
        // The balloon cannot have a target size greater than the size of
        // the guest memory.
        if u64::from(amount_mib) > mem_size_mib(self.guest_memory()) {
            return Err(BalloonError::TooManyPagesRequested);
        }

        if let Some(busdev) = self.get_bus_device(DeviceType::Virtio(TYPE_BALLOON), BALLOON_DEV_ID)
        {
            {
                let virtio_device = busdev
                    .lock()
                    .expect("Poisoned lock")
                    .mmio_transport_ref()
                    .expect("Unexpected device type")
                    .device();

                virtio_device
                    .lock()
                    .expect("Poisoned lock")
                    .as_mut_any()
                    .downcast_mut::<Balloon>()
                    .unwrap()
                    .update_size(amount_mib)?;

                Ok(())
            }
        } else {
            Err(BalloonError::DeviceNotFound)
        }
    }

    #[tracing::instrument(level = "trace", skip(self,stats_polling_interval_s))]
    /// Updates configuration for the balloon device as described in `balloon_stats_update`.
    pub fn update_balloon_stats_config(
        &mut self,
        stats_polling_interval_s: u16,
    ) -> Result<(), BalloonError> {
        if let Some(busdev) = self.get_bus_device(DeviceType::Virtio(TYPE_BALLOON), BALLOON_DEV_ID)
        {
            {
                let virtio_device = busdev
                    .lock()
                    .expect("Poisoned lock")
                    .mmio_transport_ref()
                    .expect("Unexpected device type")
                    .device();

                virtio_device
                    .lock()
                    .expect("Poisoned lock")
                    .as_mut_any()
                    .downcast_mut::<Balloon>()
                    .unwrap()
                    .update_stats_polling_interval(stats_polling_interval_s)?;
            }
            Ok(())
        } else {
            Err(BalloonError::DeviceNotFound)
        }
    }

    #[tracing::instrument(level = "trace", skip(self,exit_code))]
    /// Signals Vmm to stop and exit.
    pub fn stop(&mut self, exit_code: FcExitCode) {
        // To avoid cycles, all teardown paths take the following route:
        //   +------------------------+----------------------------+------------------------+
        //   |        Vmm             |           Action           |           Vcpu         |
        //   +------------------------+----------------------------+------------------------+
        // 1 |                        |                            | vcpu.exit(exit_code)   |
        // 2 |                        |                            | vcpu.exit_evt.write(1) |
        // 3 |                        | <--- EventFd::exit_evt --- |                        |
        // 4 | vmm.stop()             |                            |                        |
        // 5 |                        | --- VcpuEvent::Finish ---> |                        |
        // 6 |                        |                            | StateMachine::finish() |
        // 7 | VcpuHandle::join()     |                            |                        |
        // 8 | vmm.shutdown_exit_code becomes Some(exit_code) breaking the main event loop  |
        //   +------------------------+----------------------------+------------------------+
        // Vcpu initiated teardown starts from `fn Vcpu::exit()` (step 1).
        // Vmm initiated teardown starts from `pub fn Vmm::stop()` (step 4).
        // Once `vmm.shutdown_exit_code` becomes `Some(exit_code)`, it is the upper layer's
        // responsibility to break main event loop and propagate the exit code value.
        info!("Vmm is stopping.");

        // We send a "Finish" event.  If a VCPU has already exited, this is the only
        // message it will accept... but running and paused will take it as well.
        // It breaks out of the state machine loop so that the thread can be joined.
        for (idx, handle) in self.vcpus_handles.iter().enumerate() {
            if let Err(err) = handle.send_event(VcpuEvent::Finish) {
                error!("Failed to send VcpuEvent::Finish to vCPU {}: {}", idx, err);
            }
        }
        // The actual thread::join() that runs to release the thread's resource is done in
        // the VcpuHandle's Drop trait.  We can trigger that to happen now by clearing the
        // list of handles. Do it here instead of Vmm::Drop to avoid dependency cycles.
        // (Vmm's Drop will also check if this list is empty).
        self.vcpus_handles.clear();

        // Break the main event loop, propagating the Vmm exit-code.
        self.shutdown_exit_code = Some(exit_code);
    }
}

#[tracing::instrument(level = "trace", skip(vcpu_states))]
/// Process the content of the MPIDR_EL1 register in order to be able to pass it to KVM
///
/// The kernel expects to find the four affinity levels of the MPIDR in the first 32 bits of the
/// VGIC register attribute:
/// https://elixir.free-electrons.com/linux/v4.14.203/source/virt/kvm/arm/vgic/vgic-kvm-device.c#L445.
///
/// The format of the MPIDR_EL1 register is:
/// | 39 .... 32 | 31 .... 24 | 23 .... 16 | 15 .... 8 | 7 .... 0 |
/// |    Aff3    |    Other   |    Aff2    |    Aff1   |   Aff0   |
///
/// The KVM mpidr format is:
/// | 63 .... 56 | 55 .... 48 | 47 .... 40 | 39 .... 32 |
/// |    Aff3    |    Aff2    |    Aff1    |    Aff0    |
/// As specified in the linux kernel: Documentation/virt/kvm/devices/arm-vgic-v3.rst
#[cfg(target_arch = "aarch64")]
fn construct_kvm_mpidrs(vcpu_states: &[VcpuState]) -> Vec<u64> {
    vcpu_states
        .iter()
        .map(|state| {
            let cpu_affid = ((state.mpidr & 0xFF_0000_0000) >> 8) | (state.mpidr & 0xFF_FFFF);
            cpu_affid << 32
        })
        .collect()
}

impl Drop for Vmm {
    #[tracing::instrument(level = "trace", skip(self))]
    fn drop(&mut self) {
        // There are two cases when `drop()` is called:
        // 1) before the Vmm has been mutexed and subscribed to the event
        //    manager, or
        // 2) after the Vmm has been registered as a subscriber to the
        //    event manager.
        //
        // The first scenario is bound to happen if an error is raised during
        // Vmm creation (for example, during snapshot load), before the Vmm has
        // been subscribed to the event manager. If that happens, the `drop()`
        // function is called right before propagating the error. In order to
        // be able to gracefully exit Firecracker with the correct fault
        // message, we need to prepare the Vmm contents for the tear down
        // (join the vcpu threads). Explicitly calling `stop()` allows the
        // Vmm to be successfully dropped and firecracker to propagate the
        // error.
        //
        // In the second case, before dropping the Vmm object, the event
        // manager calls `stop()`, which sends a `Finish` event to the vcpus
        // and joins the vcpu threads. The Vmm is dropped after everything is
        // ready to be teared down. The line below is a no-op, because the Vmm
        // has already been stopped by the event manager at this point.
        self.stop(self.shutdown_exit_code.unwrap_or(FcExitCode::Ok));

        if let Some(observer) = self.events_observer.as_mut() {
            let res = observer.lock().set_canon_mode().map_err(|err| {
                warn!("Cannot set canonical mode for the terminal. {:?}", err);
                err
            });
            if let Err(err) = res {
                warn!("{}", VmmError::VmmObserverTeardown(err));
            }
        }

        // Write the metrics before exiting.
        if let Err(err) = METRICS.write() {
            error!("Failed to write metrics while stopping: {}", err);
        }

        if !self.vcpus_handles.is_empty() {
            error!("Failed to tear down Vmm: the vcpu threads have not finished execution.");
        }
    }
}

impl MutEventSubscriber for Vmm {
    #[tracing::instrument(level = "trace", skip(self,event))]
    /// Handle a read event (EPOLLIN).
    fn process(&mut self, event: Events, _: &mut EventOps) {
        let source = event.fd();
        let event_set = event.event_set();

        if source == self.vcpus_exit_evt.as_raw_fd() && event_set == EventSet::IN {
            // Exit event handling should never do anything more than call 'self.stop()'.
            let _ = self.vcpus_exit_evt.read();

            let mut exit_code = None;
            // Query each vcpu for their exit_code.
            for handle in &self.vcpus_handles {
                match handle.response_receiver().try_recv() {
                    Ok(VcpuResponse::Exited(status)) => {
                        exit_code = Some(status);
                        // Just use the first encountered exit-code.
                        break;
                    }
                    Ok(_response) => {} // Don't care about these, we are exiting.
                    Err(TryRecvError::Empty) => {} // Nothing pending in channel
                    Err(err) => {
                        panic!("Error while looking for VCPU exit status: {}", err);
                    }
                }
            }
            self.stop(exit_code.unwrap_or(FcExitCode::Ok));
        } else {
            error!("Spurious EventManager event for handler: Vmm");
        }
    }

    #[tracing::instrument(level = "trace", skip(self,ops))]
    fn init(&mut self, ops: &mut EventOps) {
        if let Err(err) = ops.add(Events::new(&self.vcpus_exit_evt, EventSet::IN)) {
            error!("Failed to register vmm exit event: {}", err);
        }
    }
}
