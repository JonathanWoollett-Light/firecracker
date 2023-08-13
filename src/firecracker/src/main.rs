// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

mod api_server_adapter;
mod metrics;
mod seccomp;

use std::fs::{self, File};
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{io, panic};

use api_server_adapter::ApiServerError;
use event_manager::SubscriberOps;
use logger::{error, info, ProcessTimeReporter, StoreMetric, METRICS};
use seccomp::FilterError;
use seccompiler::BpfThreadMap;
use snapshot::{Error as SnapshotError, Snapshot};
use utils::arg_parser::{ArgParser, Argument};
use utils::terminal::Terminal;
use utils::validators::validate_instance_id;
use vmm::builder::StartMicrovmError;
use vmm::resources::VmResources;
use vmm::signal_handler::register_signal_handlers;
use vmm::version_map::{FC_VERSION_TO_SNAP_VERSION, VERSION_MAP};
use vmm::vmm_config::instance_info::{InstanceInfo, VmState};
use vmm::vmm_config::logger::LevelFilter;
use vmm::vmm_config::metrics::{init_metrics, MetricsConfig};
use vmm::{EventManager, FcExitCode, HTTP_MAX_PAYLOAD_SIZE};

use crate::seccomp::SeccompConfig;

// The reason we place default API socket under /run is that API socket is a
// runtime file.
// see https://refspecs.linuxfoundation.org/FHS_3.0/fhs/ch03s15.html for more information.
const DEFAULT_API_SOCK_PATH: &str = "/run/firecracker.socket";
const FIRECRACKER_VERSION: &str = env!("FIRECRACKER_VERSION");
const MMDS_CONTENT_ARG: &str = "metadata";

#[derive(Debug, thiserror::Error)]
enum MainError {
    #[error("Failed to register signal handlers: {0}")]
    RegisterSignalHandlers(#[source] utils::errno::Error),
    #[error("Arguments parsing error: {0} \n\nFor more information try --help.")]
    ParseArguments(#[from] utils::arg_parser::Error),
    #[error("When printing Snapshot Data format: {0}")]
    PrintSnapshotDataFormat(#[from] SnapshotVersionError),
    #[error(
        "Invalid value for logger level: {0}.Possible values: [Off, Error, Warning, Info, Debug, \
         Trace]"
    )]
    InvalidLogLevel(<LevelFilter as FromStr>::Err),
    #[error("Failed to deserialize log filter: {0}")]
    DeserializeLogFilter(serde_json::Error),
    #[error("Could not initialize logger: {0}")]
    LoggerInitialization(vmm::vmm_config::logger::InitLoggerError),
    #[error("Could not initialize metrics: {0:?}")]
    MetricsInitialization(vmm::vmm_config::metrics::MetricsConfigError),
    #[error("Seccomp error: {0}")]
    SeccompFilter(FilterError),
    #[error("RunWithApiError error: {0}")]
    RunWithApi(ApiServerError),
    #[error("RunWithoutApiError error: {0}")]
    RunWithoutApiError(RunWithoutApiError),
}

impl From<MainError> for ExitCode {
    fn from(value: MainError) -> Self {
        let exit_code = match value {
            MainError::ParseArguments(_) => FcExitCode::ArgParsing,
            MainError::InvalidLogLevel(_) => FcExitCode::BadConfiguration,
            MainError::RunWithApi(ApiServerError::MicroVMStoppedWithoutError(code)) => code,
            MainError::RunWithApi(ApiServerError::MicroVMStoppedWithError(code)) => code,
            _ => FcExitCode::GenericError,
        };

        ExitCode::from(exit_code as u8)
    }
}

fn main() -> ExitCode {
    let result = main_exec();
    if let Err(err) = result {
        error!("{err}");
        eprintln!("Error: {err:?}");
        ExitCode::from(err)
    } else {
        ExitCode::SUCCESS
    }
}

fn main_exec() -> Result<(), MainError> {
    register_signal_handlers().map_err(MainError::RegisterSignalHandlers)?;

    #[cfg(target_arch = "aarch64")]
    enable_ssbd_mitigation();

    // We need this so that we can reset terminal to canonical mode if panic occurs.
    let stdin = io::stdin();

    // Start firecracker by setting up a panic hook, which will be called before
    // terminating as we're building with panic = "abort".
    // It's worth noting that the abort is caused by sending a SIG_ABORT signal to the process.
    panic::set_hook(Box::new(move |info| {
        // We're currently using the closure parameter, which is a &PanicInfo, for printing the
        // origin of the panic, including the payload passed to panic! and the source code location
        // from which the panic originated.
        error!("Firecracker {}", info);
        if let Err(err) = stdin.lock().set_canon_mode() {
            error!(
                "Failure while trying to reset stdin to canonical mode: {}",
                err
            );
        }

        METRICS.vmm.panic_count.store(1);

        // Write the metrics before aborting.
        if let Err(err) = METRICS.write() {
            error!("Failed to write metrics while panicking: {}", err);
        }
    }));

    let http_max_payload_size_str = HTTP_MAX_PAYLOAD_SIZE.to_string();

    let mut arg_parser =
        ArgParser::new()
            .arg(
                Argument::new("api-sock")
                    .takes_value(true)
                    .default_value(DEFAULT_API_SOCK_PATH)
                    .help("Path to unix domain socket used by the API."),
            )
            .arg(
                Argument::new("id")
                    .takes_value(true)
                    .default_value("anonymous-instance")
                    .help("MicroVM unique identifier."),
            )
            .arg(
                Argument::new("seccomp-filter")
                    .takes_value(true)
                    .forbids(vec!["no-seccomp"])
                    .help(
                        "Optional parameter which allows specifying the path to a custom seccomp \
                         filter. For advanced users.",
                    ),
            )
            .arg(
                Argument::new("no-seccomp")
                    .takes_value(false)
                    .forbids(vec!["seccomp-filter"])
                    .help(
                        "Optional parameter which allows starting and using a microVM without \
                         seccomp filtering. Not recommended.",
                    ),
            )
            .arg(
                Argument::new("start-time-us").takes_value(true).help(
                    "Process start time (wall clock, microseconds). This parameter is optional.",
                ),
            )
            .arg(Argument::new("start-time-cpu-us").takes_value(true).help(
                "Process start CPU time (wall clock, microseconds). This parameter is optional.",
            ))
            .arg(Argument::new("parent-cpu-time-us").takes_value(true).help(
                "Parent process CPU time (wall clock, microseconds). This parameter is optional.",
            ))
            .arg(
                Argument::new("config-file")
                    .takes_value(true)
                    .help("Path to a file that contains the microVM configuration in JSON format."),
            )
            .arg(
                Argument::new(MMDS_CONTENT_ARG).takes_value(true).help(
                    "Path to a file that contains metadata in JSON format to add to the mmds.",
                ),
            )
            .arg(
                Argument::new("start-time-us").takes_value(true).help(
                    "Process start time (wall clock, microseconds). This parameter is optional.",
                ),
            )
            .arg(Argument::new("start-time-cpu-us").takes_value(true).help(
                "Process start CPU time (wall clock, microseconds). This parameter is optional.",
            ))
            .arg(Argument::new("parent-cpu-time-us").takes_value(true).help(
                "Parent process CPU time (wall clock, microseconds). This parameter is optional.",
            ))
            .arg(
                Argument::new("config-file")
                    .takes_value(true)
                    .help("Path to a file that contains the microVM configuration in JSON format."),
            )
            .arg(
                Argument::new(MMDS_CONTENT_ARG).takes_value(true).help(
                    "Path to a file that contains metadata in JSON format to add to the mmds.",
                ),
            )
            .arg(
                Argument::new("no-api")
                    .takes_value(false)
                    .requires("config-file")
                    .help(
                        "Optional parameter which allows starting and using a microVM without an \
                         active API socket.",
                    ),
            )
            .arg(
                Argument::new("log-path")
                    .takes_value(true)
                    .help("Path to a fifo or a file used for configuring the logger on startup."),
            )
            .arg(
                Argument::new("log-filter")
                    .takes_value(true)
                    .help("Filter for logging, if set overrides `level`."),
            )
            .arg(
                Argument::new("level")
                    .takes_value(true)
                    .help("Set the logger level."),
            )
            .arg(
                Argument::new("show-level")
                    .takes_value(false)
                    .help("Whether or not to output the level in the logs."),
            )
            .arg(Argument::new("show-log-origin").takes_value(false).help(
                "Whether or not to include the file path and line number of the log's origin.",
            ))
            .arg(
                Argument::new("metrics-path")
                    .takes_value(true)
                    .help("Path to a fifo or a file used for configuring the metrics on startup."),
            )
            .arg(Argument::new("boot-timer").takes_value(false).help(
                "Whether or not to load boot timer device for logging elapsed time since \
                 InstanceStart command.",
            ))
            .arg(Argument::new("version").takes_value(false).help(
                "Print the binary version number and a list of supported snapshot data format \
                 versions.",
            ))
            .arg(
                Argument::new("describe-snapshot")
                    .takes_value(true)
                    .help("Print the data format version of the provided snapshot state file."),
            )
            .arg(
                Argument::new("http-api-max-payload-size")
                    .takes_value(true)
                    .default_value(&http_max_payload_size_str)
                    .help("Http API request payload max size, in bytes."),
            )
            .arg(
                Argument::new("mmds-size-limit")
                    .takes_value(true)
                    .help("Mmds data store limit, in bytes."),
            );

    arg_parser.parse_from_cmdline()?;
    let arguments = arg_parser.arguments();

    if arguments.flag_present("help") {
        println!("Firecracker v{}\n", FIRECRACKER_VERSION);
        println!("{}", arg_parser.formatted_help());
        return Ok(());
    }

    if arguments.flag_present("version") {
        println!("Firecracker v{}\n", FIRECRACKER_VERSION);
        print_supported_snapshot_versions();
        return Ok(());
    }

    if let Some(snapshot_path) = arguments.single_value("describe-snapshot") {
        print_snapshot_data_format(snapshot_path)?;
        return Ok(());
    }

    // Display warnings for any used deprecated parameters.
    // Currently unused since there are no deprecated parameters. Uncomment the line when
    // deprecating one.
    // warn_deprecated_parameters(&arguments);

    // It's safe to unwrap here because the field's been provided with a default value.
    let instance_id = arguments.single_value("id").unwrap();
    validate_instance_id(instance_id.as_str()).expect("Invalid instance ID");

    let instance_info = InstanceInfo {
        id: instance_id.clone(),
        state: VmState::NotStarted,
        vmm_version: FIRECRACKER_VERSION.to_string(),
        app_name: "Firecracker".to_string(),
    };

    // Configure logger, the logger handles can be used to re-configure the logger with the API.
    let logger_handles = {
        let level_res = arguments
            .single_value("level")
            .map(|s| LevelFilter::from_str(s))
            .transpose();
        let level = level_res.map_err(MainError::InvalidLogLevel)?;

        let filter = if let Some(log_filter) = arguments.single_value("log-filter") {
            Some(serde_json::from_str(log_filter).map_err(MainError::DeserializeLogFilter)?)
        } else {
            None
        };
        let logger_config = vmm::vmm_config::logger::LoggerConfig {
            log_path: arguments.single_value("log-path").map(PathBuf::from),
            level,
            show_level: Some(arguments.flag_present("show-level")),
            show_log_origin: Some(arguments.flag_present("show-log-origin")),
            filter,
        };
        logger_config
            .init()
            .map_err(MainError::LoggerInitialization)?
    };

    if let Some(metrics_path) = arguments.single_value("metrics-path") {
        let metrics_config = MetricsConfig {
            metrics_path: PathBuf::from(metrics_path),
        };
        init_metrics(metrics_config).map_err(MainError::MetricsInitialization)?;
    }

    let mut seccomp_filters: BpfThreadMap = SeccompConfig::from_args(
        arguments.flag_present("no-seccomp"),
        arguments.single_value("seccomp-filter"),
    )
    .and_then(seccomp::get_filters)
    .map_err(MainError::SeccompFilter)?;

    let vmm_config_json = arguments
        .single_value("config-file")
        .map(fs::read_to_string)
        .map(|x| x.expect("Unable to open or read from the configuration file"));

    let metadata_json = arguments
        .single_value(MMDS_CONTENT_ARG)
        .map(fs::read_to_string)
        .map(|x| x.expect("Unable to open or read from the mmds content file"));

    let boot_timer_enabled = arguments.flag_present("boot-timer");
    let api_enabled = !arguments.flag_present("no-api");
    let api_payload_limit = arg_parser
        .arguments()
        .single_value("http-api-max-payload-size")
        .map(|lim| {
            lim.parse::<usize>()
                .expect("'http-api-max-payload-size' parameter expected to be of 'usize' type.")
        })
        // Safe to unwrap as we provide a default value.
        .unwrap();

    // If the mmds size limit is not explicitly configured, default to using the
    // `http-api-max-payload-size` value.
    let mmds_size_limit = arg_parser
        .arguments()
        .single_value("mmds-size-limit")
        .map(|lim| {
            lim.parse::<usize>()
                .expect("'mmds-size-limit' parameter expected to be of 'usize' type.")
        })
        .unwrap_or_else(|| api_payload_limit);

    if api_enabled {
        let bind_path = arguments
            .single_value("api-sock")
            .map(PathBuf::from)
            .expect("Missing argument: api-sock");

        let start_time_us = arguments.single_value("start-time-us").map(|s| {
            s.parse::<u64>()
                .expect("'start-time-us' parameter expected to be of 'u64' type.")
        });

        let start_time_cpu_us = arguments.single_value("start-time-cpu-us").map(|s| {
            s.parse::<u64>()
                .expect("'start-time-cpu-us' parameter expected to be of 'u64' type.")
        });

        let parent_cpu_time_us = arguments.single_value("parent-cpu-time-us").map(|s| {
            s.parse::<u64>()
                .expect("'parent-cpu-time-us' parameter expected to be of 'u64' type.")
        });

        let process_time_reporter =
            ProcessTimeReporter::new(start_time_us, start_time_cpu_us, parent_cpu_time_us);

        api_server_adapter::run_with_api(
            &mut seccomp_filters,
            vmm_config_json,
            bind_path,
            instance_info,
            process_time_reporter,
            boot_timer_enabled,
            api_payload_limit,
            mmds_size_limit,
            metadata_json.as_deref(),
            logger_handles,
        )
        .map_err(MainError::RunWithApi)
    } else {
        let seccomp_filters: BpfThreadMap = seccomp_filters
            .into_iter()
            .filter(|(k, _)| k != "api")
            .collect();
        run_without_api(
            &seccomp_filters,
            vmm_config_json,
            instance_info,
            boot_timer_enabled,
            mmds_size_limit,
            metadata_json.as_deref(),
        )
        .map_err(MainError::RunWithoutApiError)
    }
}

/// Enable SSBD mitigation through `prctl`.
#[cfg(target_arch = "aarch64")]
pub fn enable_ssbd_mitigation() {
    // Parameters for `prctl`
    // TODO: generate bindings for these from the kernel sources.
    // https://elixir.bootlin.com/linux/v4.17/source/include/uapi/linux/prctl.h#L212
    const PR_SET_SPECULATION_CTRL: i32 = 53;
    const PR_SPEC_STORE_BYPASS: u64 = 0;
    const PR_SPEC_FORCE_DISABLE: u64 = 1u64 << 3;

    // SAFETY: Parameters are valid since they are copied verbatim
    // from the kernel's UAPI.
    // PR_SET_SPECULATION_CTRL only uses those 2 parameters, so it's ok
    // to leave the latter 2 as zero.
    let ret = unsafe {
        libc::prctl(
            PR_SET_SPECULATION_CTRL,
            PR_SPEC_STORE_BYPASS,
            PR_SPEC_FORCE_DISABLE,
            0,
            0,
        )
    };

    if ret < 0 {
        let last_error = std::io::Error::last_os_error().raw_os_error().unwrap();
        error!(
            "Could not enable SSBD mitigation through prctl, error {}",
            last_error
        );
        if last_error == libc::EINVAL {
            error!("The host does not support SSBD mitigation through prctl.");
        }
    }
}

// Log a warning for any usage of deprecated parameters.
#[allow(unused)]
fn warn_deprecated_parameters() {}

// Print supported snapshot data format versions.
fn print_supported_snapshot_versions() {
    let mut versions: Vec<_> = FC_VERSION_TO_SNAP_VERSION
        .iter()
        .map(|(key, _)| key.clone())
        .collect();
    versions.sort();

    println!("Supported snapshot data format versions:");
    for v in versions.iter() {
        println!("{v}");
    }
}

#[derive(Debug, thiserror::Error)]
enum SnapshotVersionError {
    #[error("Unable to open snapshot state file: {0}")]
    OpenSnapshot(io::Error),
    #[error("Invalid data format version of snapshot file: {0}")]
    SnapshotVersion(SnapshotError),
    #[error("Cannot translate snapshot data version {0} to Firecracker microVM version")]
    FirecrackerVersion(u16),
}

// Print data format of provided snapshot state file.
fn print_snapshot_data_format(snapshot_path: &str) -> Result<(), SnapshotVersionError> {
    let mut snapshot_reader =
        File::open(snapshot_path).map_err(SnapshotVersionError::OpenSnapshot)?;

    let data_format_version = Snapshot::get_data_version(&mut snapshot_reader, &VERSION_MAP)
        .map_err(SnapshotVersionError::SnapshotVersion)?;

    let (key, _) = FC_VERSION_TO_SNAP_VERSION
        .iter()
        .find(|(_, &val)| val == data_format_version)
        .ok_or_else(|| SnapshotVersionError::FirecrackerVersion(data_format_version))?;

    println!("v{}", key);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum BuildFromJsonError {
    #[error("Configuration for VMM from one single json failed: {0}")]
    ParseFromJson(vmm::resources::ResourcesError),
    #[error("Could not Start MicroVM from one single json: {0}")]
    StartMicroVM(StartMicrovmError),
}

// Configure and start a microVM as described by the command-line JSON.
fn build_microvm_from_json(
    seccomp_filters: &BpfThreadMap,
    event_manager: &mut EventManager,
    config_json: String,
    instance_info: InstanceInfo,
    boot_timer_enabled: bool,
    mmds_size_limit: usize,
    metadata_json: Option<&str>,
) -> Result<(VmResources, Arc<Mutex<vmm::Vmm>>), BuildFromJsonError> {
    let mut vm_resources =
        VmResources::from_json(&config_json, &instance_info, mmds_size_limit, metadata_json)
            .map_err(BuildFromJsonError::ParseFromJson)?;
    vm_resources.boot_timer = boot_timer_enabled;
    let vmm = vmm::builder::build_and_boot_microvm(
        &instance_info,
        &vm_resources,
        event_manager,
        seccomp_filters,
    )
    .map_err(BuildFromJsonError::StartMicroVM)?;

    info!("Successfully started microvm that was configured from one single json");

    Ok((vm_resources, vmm))
}

#[derive(Debug, thiserror::Error)]
enum RunWithoutApiError {
    #[error("MicroVMStopped without an error: {0:?}")]
    Shutdown(FcExitCode),
    #[error("Failed to build MicroVM from Json: {0}")]
    BuildMicroVMFromJson(BuildFromJsonError),
}

fn run_without_api(
    seccomp_filters: &BpfThreadMap,
    config_json: Option<String>,
    instance_info: InstanceInfo,
    bool_timer_enabled: bool,
    mmds_size_limit: usize,
    metadata_json: Option<&str>,
) -> Result<(), RunWithoutApiError> {
    let mut event_manager = EventManager::new().expect("Unable to create EventManager");

    // Create the firecracker metrics object responsible for periodically printing metrics.
    let firecracker_metrics = Arc::new(Mutex::new(metrics::PeriodicMetrics::new()));
    event_manager.add_subscriber(firecracker_metrics.clone());

    // Build the microVm. We can ignore VmResources since it's not used without api.
    let (_, vmm) = build_microvm_from_json(
        seccomp_filters,
        &mut event_manager,
        // Safe to unwrap since '--no-api' requires this to be set.
        config_json.unwrap(),
        instance_info,
        bool_timer_enabled,
        mmds_size_limit,
        metadata_json,
    )
    .map_err(RunWithoutApiError::BuildMicroVMFromJson)?;

    // Start the metrics.
    firecracker_metrics
        .lock()
        .expect("Poisoned lock")
        .start(metrics::WRITE_METRICS_PERIOD_MS);

    // Run the EventManager that drives everything in the microVM.
    loop {
        event_manager
            .run()
            .expect("Failed to start the event manager");

        if let Some(exit_code) = vmm.lock().unwrap().shutdown_exit_code() {
            return Err(RunWithoutApiError::Shutdown(exit_code));
        }
    }
}
