// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use core::fmt;
use std::convert::{From, TryInto};
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, LineWriter, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Mutex;

use libc::O_NONBLOCK;
use rate_limiter::{BucketUpdate, RateLimiter, TokenBucket};
use serde::{Deserialize, Serialize};
use tracing::{Collect, Event};
use tracing_flame::FlameSubscriber;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt::format::{self, FormatEvent, FormatFields};
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::registry::{LookupSpan, Registry};
use tracing_subscriber::reload::Handle;
use tracing_subscriber::subscribe::{CollectExt, Layered};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::FmtSubscriber;

/// Wrapper for configuring the balloon device.
pub mod balloon;
/// Wrapper for configuring the microVM boot source.
pub mod boot_source;
/// Wrapper for configuring the block devices.
pub mod drive;
/// Wrapper for configuring the entropy device attached to the microVM.
pub mod entropy;
/// Wrapper over the microVM general information attached to the microVM.
pub mod instance_info;
/// Wrapper for configuring the memory and CPU of the microVM.
pub mod machine_config;
/// Wrapper for configuring the metrics.
pub mod metrics;
/// Wrapper for configuring the MMDS.
pub mod mmds;
/// Wrapper for configuring the network devices attached to the microVM.
pub mod net;
/// Wrapper for configuring microVM snapshots and the microVM state.
pub mod snapshot;
/// Wrapper for configuring the vsock devices attached to the microVM.
pub mod vsock;

// TODO: Migrate the VMM public-facing code (i.e. interface) to use stateless structures,
// for receiving data/args, such as the below `RateLimiterConfig` and `TokenBucketConfig`.
// Also todo: find a better suffix than `Config`; it should illustrate the static nature
// of the enclosed data.
// Currently, data is passed around using live/stateful objects. Switching to static/stateless
// objects will simplify both the ownership model and serialization.
// Public access would then be more tightly regulated via `VmmAction`s, consisting of tuples like
// (entry-point-into-VMM-logic, stateless-args-structure).

/// A public-facing, stateless structure, holding all the data we need to create a TokenBucket
/// (live) object.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct TokenBucketConfig {
    /// See TokenBucket::size.
    pub size: u64,
    /// See TokenBucket::one_time_burst.
    pub one_time_burst: Option<u64>,
    /// See TokenBucket::refill_time.
    pub refill_time: u64,
}

impl From<&TokenBucket> for TokenBucketConfig {
    fn from(tb: &TokenBucket) -> Self {
        let one_time_burst = match tb.initial_one_time_burst() {
            0 => None,
            v => Some(v),
        };
        TokenBucketConfig {
            size: tb.capacity(),
            one_time_burst,
            refill_time: tb.refill_time_ms(),
        }
    }
}

/// A public-facing, stateless structure, holding all the data we need to create a RateLimiter
/// (live) object.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RateLimiterConfig {
    /// Data used to initialize the RateLimiter::bandwidth bucket.
    pub bandwidth: Option<TokenBucketConfig>,
    /// Data used to initialize the RateLimiter::ops bucket.
    pub ops: Option<TokenBucketConfig>,
}

/// A public-facing, stateless structure, specifying RateLimiter properties updates.
#[derive(Debug)]
pub struct RateLimiterUpdate {
    /// Possible update to the RateLimiter::bandwidth bucket.
    pub bandwidth: BucketUpdate,
    /// Possible update to the RateLimiter::ops bucket.
    pub ops: BucketUpdate,
}

fn get_bucket_update(tb_cfg: &Option<TokenBucketConfig>) -> BucketUpdate {
    match tb_cfg {
        // There is data to update.
        Some(tb_cfg) => {
            TokenBucket::new(
                tb_cfg.size,
                tb_cfg.one_time_burst.unwrap_or(0),
                tb_cfg.refill_time,
            )
            // Updated active rate-limiter.
            .map(BucketUpdate::Update)
            // Updated/deactivated rate-limiter
            .unwrap_or(BucketUpdate::Disabled)
        }
        // No update to the rate-limiter.
        None => BucketUpdate::None,
    }
}

impl From<Option<RateLimiterConfig>> for RateLimiterUpdate {
    fn from(cfg: Option<RateLimiterConfig>) -> Self {
        if let Some(cfg) = cfg {
            RateLimiterUpdate {
                bandwidth: get_bucket_update(&cfg.bandwidth),
                ops: get_bucket_update(&cfg.ops),
            }
        } else {
            // No update to the rate-limiter.
            RateLimiterUpdate {
                bandwidth: BucketUpdate::None,
                ops: BucketUpdate::None,
            }
        }
    }
}

impl TryInto<RateLimiter> for RateLimiterConfig {
    type Error = io::Error;
    fn try_into(self) -> Result<RateLimiter, Self::Error> {
        let bw = self.bandwidth.unwrap_or_default();
        let ops = self.ops.unwrap_or_default();
        RateLimiter::new(
            bw.size,
            bw.one_time_burst.unwrap_or(0),
            bw.refill_time,
            ops.size,
            ops.one_time_burst.unwrap_or(0),
            ops.refill_time,
        )
    }
}

impl From<&RateLimiter> for RateLimiterConfig {
    fn from(rl: &RateLimiter) -> Self {
        RateLimiterConfig {
            bandwidth: rl.bandwidth().map(TokenBucketConfig::from),
            ops: rl.ops().map(TokenBucketConfig::from),
        }
    }
}

impl RateLimiterConfig {
    // Option<T> already implements From<T> so we have to use a custom one.
    fn into_option(self) -> Option<RateLimiterConfig> {
        if self.bandwidth.is_some() || self.ops.is_some() {
            Some(self)
        } else {
            None
        }
    }
}

/// Create and opens a File for writing to it.
/// In case we open a FIFO, in order to not block the instance if nobody is consuming the message
/// that is flushed to the two pipes, we are opening it with `O_NONBLOCK` flag.
/// In this case, writing to a pipe will start failing when reaching 64K of unconsumed content.
fn open_file_nonblock(path: &Path) -> Result<File, std::io::Error> {
    OpenOptions::new()
        .custom_flags(O_NONBLOCK)
        .read(true)
        .write(true)
        .open(path)
}

// TODO: See below doc comment.
/// Mimic of `log::Level`.
///
/// This is used instead of `log::Level` to support:
/// 1. Aliasing `Warn` as `Warning` to avoid a breaking change in the API (which previously only
///    accepted `Warning`).
/// 2. Setting the default to `Warn` to avoid a breaking change.
///
/// This alias, custom `Default` and type should be removed in the next breaking update to simplify
/// the code and API (and `log::Level` should be used in place).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Level {
    /// The “error” level.
    ///
    /// Designates very serious errors.
    #[serde(alias = "ERROR")]
    Error,
    /// The “warn” level.
    ///
    /// Designates hazardous situations.
    #[serde(alias = "WARNING", alias = "Warning")]
    Warn,
    /// The “info” level.
    ///
    /// Designates useful information.
    #[serde(alias = "INFO")]
    Info,
    /// The “debug” level.
    ///
    /// Designates lower priority information.
    #[serde(alias = "DEBUG")]
    Debug,
    /// The “trace” level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    #[serde(alias = "TRACE")]
    Trace,
}
impl Default for Level {
    fn default() -> Self {
        Self::Warn
    }
}
impl From<Level> for tracing::Level {
    fn from(level: Level) -> tracing::Level {
        match level {
            Level::Error => tracing::Level::ERROR,
            Level::Warn => tracing::Level::WARN,
            Level::Info => tracing::Level::INFO,
            Level::Debug => tracing::Level::DEBUG,
            Level::Trace => tracing::Level::TRACE,
        }
    }
}
impl From<log::Level> for Level {
    fn from(level: log::Level) -> Level {
        match level {
            log::Level::Error => Level::Error,
            log::Level::Warn => Level::Warn,
            log::Level::Info => Level::Info,
            log::Level::Debug => Level::Debug,
            log::Level::Trace => Level::Trace,
        }
    }
}
impl FromStr for Level {
    type Err = <log::Level as FromStr>::Err;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // This is required to avoid a breaking change.
        match s {
            "ERROR" => Ok(Level::Error),
            "WARNING" | "Warning" => Ok(Level::Warn),
            "INFO" => Ok(Level::Info),
            "DEBUG" => Ok(Level::Debug),
            "TRACE" => Ok(Level::Trace),
            _ => log::Level::from_str(s).map(Level::from),
        }
    }
}

/// Strongly typed structure used to describe the logger.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoggerConfig {
    /// Named pipe or file used as output for logs.
    pub log_path: Option<PathBuf>,
    /// The level of the Logger.
    pub level: Option<Level>,
    /// When enabled, the logger will append to the output the severity of the log entry.
    pub show_level: Option<bool>,
    /// When enabled, the logger will append the origin of the log entry.
    pub show_log_origin: Option<bool>,
    /// When enabled, the logger will use the default [`tracing_subscriber::fmt::format::Format`]
    /// formatter.
    pub new_format: Option<bool>,
    /// The profile file to output.
    pub profile_file: Option<PathBuf>,
}

/// Error with actions on the `LoggerConfig`.
#[derive(Debug, thiserror::Error)]
pub enum LoggerConfigError {
    /// Failed to initialize logger.
    #[error("Failed to initialize logger: {0}")]
    Init(tracing_subscriber::util::TryInitError),
    /// Failed to open target file.
    #[error("Failed to open target file: {0}")]
    File(std::io::Error),
    /// Failed to write initialization message.
    #[error("Failed to write initialization message: {0}")]
    Write(std::io::Error),
    /// Failed to initialize flame layer.
    #[error("Failed to   flame layer.")]
    Flame,
}

type ReloadSubscriber<S> = tracing_subscriber::reload::Subscriber<S>;
type FmtInner = Layered<ReloadSubscriber<LevelFilter>, Layered<EnvFilter, Registry>>;
type FmtType = FmtSubscriber<FmtInner, format::DefaultFields, LoggerFormatter, BoxMakeWriter>;
type FlameInner = Layered<ReloadSubscriber<FmtType>, FmtInner>;
type FlameType = FlameSubscriber<FlameInner, FlameWriter>;

// TODO Remove `C` as a generic.
/// Handles that allow re-configuring the logger.
#[derive(Debug)]
pub struct LoggerHandles {
    filter: Handle<LevelFilter>,
    fmt: Handle<FmtType>,
    flame: Handle<FlameType>,
}

#[derive(Debug)]
enum FlameWriter {
    Sink(std::io::Sink),
    File(BufWriter<std::fs::File>),
}
impl Write for FlameWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Sink(sink) => sink.write(buf),
            Self::File(file) => file.write(buf),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Sink(sink) => sink.flush(),
            Self::File(file) => file.flush(),
        }
    }
}

impl LoggerConfig {
    /// Initializes the logger.
    ///
    /// Returns handles that can be used to dynamically re-configure the logger.
    pub fn init(&self) -> Result<LoggerHandles, LoggerConfigError> {
        // Setup filter
        let (filter, filter_handle) = {
            let level = tracing::Level::from(self.level.unwrap_or_default());
            let filter_subscriber = LevelFilter::from_level(level);
            ReloadSubscriber::new(filter_subscriber)
        };

        // Setup fmt layer
        let (fmt, fmt_handle) = {
            let fmt_writer = match &self.log_path {
                Some(path) => {
                    // TODO Can we add `.create(true)` so the user doesn't need to pre-create the
                    // log file? In case we open a FIFO, in order to not block
                    // the instance if nobody is consuming the message that is
                    // flushed to the two pipes, we are opening it with `O_NONBLOCK` flag.
                    // In this case, writing to a pipe will start failing when reaching 64K of
                    // unconsumed content.
                    let file = std::fs::OpenOptions::new()
                        .custom_flags(libc::O_NONBLOCK)
                        .read(true)
                        .write(true)
                        .open(path)
                        .map_err(LoggerConfigError::File)?;
                    // Wrap file to satisfy `tracing_subscriber::fmt::MakeWriter`.
                    let writer = Mutex::new(LineWriter::new(file));
                    BoxMakeWriter::new(writer)
                }
                None => BoxMakeWriter::new(std::io::stdout),
            };
            let fmt_subscriber = FmtSubscriber::new()
                .event_format(LoggerFormatter::new(
                    self.new_format.unwrap_or_default(),
                    self.show_level.unwrap_or_default(),
                    self.show_log_origin.unwrap_or_default(),
                ))
                .with_writer(fmt_writer);
            ReloadSubscriber::new(fmt_subscriber)
        };

        // Setup flame layer
        let (flame, flame_handle) = {
            let flame_writer = match &self.profile_file {
                Some(file) => FlameWriter::File(BufWriter::new(
                    std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(file)
                        .unwrap(),
                )),
                None => FlameWriter::Sink(std::io::sink()),
            };
            let flame_subscriber = FlameSubscriber::new(flame_writer);
            ReloadSubscriber::new(flame_subscriber)
        };

        // Setup the env layer
        let env = EnvFilter::builder()
            .with_default_directive(LevelFilter::TRACE.into())
            .from_env_lossy();

        Registry::default()
            .with(env)
            .with(filter)
            .with(fmt)
            .with(flame)
            .try_init()
            .map_err(LoggerConfigError::Init)?;

        tracing::error!("Error level logs enabled.");
        tracing::warn!("Warn level logs enabled.");
        tracing::info!("Info level logs enabled.");
        tracing::debug!("Debug level logs enabled.");
        tracing::trace!("Trace level logs enabled.");

        Ok(LoggerHandles {
            filter: filter_handle,
            fmt: fmt_handle,
            flame: flame_handle,
        })
    }
    /// Updates the logger using the given handles.
    pub fn update(&self, LoggerHandles { filter, fmt, flame }: &LoggerHandles) {
        // Update the log path
        if let Some(log_path) = &self.log_path {
            // In case we open a FIFO, in order to not block the instance if nobody is consuming the
            // message that is flushed to the two pipes, we are opening it with `O_NONBLOCK` flag.
            // In this case, writing to a pipe will start failing when reaching 64K of unconsumed
            // content.
            let file = std::fs::OpenOptions::new()
                .custom_flags(libc::O_NONBLOCK)
                .read(true)
                .write(true)
                .open(log_path)
                .map_err(LoggerConfigError::File)
                .unwrap();

            fmt.modify(|f| *f.writer_mut() = BoxMakeWriter::new(Mutex::new(file)))
                .unwrap();
        }

        // Update the filter level
        if let Some(level) = self.level {
            filter
                .modify(|f| *f = LevelFilter::from_level(tracing::Level::from(level)))
                .unwrap();
        }

        // Update if the logger shows the level
        if let Some(show_level) = self.show_level {
            SHOW_LEVEL.store(show_level, SeqCst);
        }

        // Updates if the logger shows the origin
        if let Some(show_log_origin) = self.show_log_origin {
            SHOW_LOG_ORIGIN.store(show_log_origin, SeqCst);
        }

        // Updates if the logger uses the new format
        if let Some(new_format) = self.new_format {
            NEW_FORMAT.store(new_format, SeqCst);
        }

        // Reload the flame layer with a new target file
        if let Some(profile_file) = &self.profile_file {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(profile_file)
                .unwrap();
            flame
                .reload(FlameSubscriber::new(FlameWriter::File(BufWriter::new(
                    file,
                ))))
                .unwrap();
        }
    }
}

#[derive(Debug)]
struct LoggerFormatter;
impl LoggerFormatter {
    pub fn new(new_format: bool, show_level: bool, show_log_origin: bool) -> Self {
        NEW_FORMAT.store(new_format, SeqCst);
        SHOW_LEVEL.store(show_level, SeqCst);
        SHOW_LOG_ORIGIN.store(show_log_origin, SeqCst);
        Self
    }
}

// TODO Using statics for these options is a bad solution, don't do this.
static NEW_FORMAT: AtomicBool = AtomicBool::new(false);
static SHOW_LEVEL: AtomicBool = AtomicBool::new(false);
static SHOW_LOG_ORIGIN: AtomicBool = AtomicBool::new(false);

impl<S, N> FormatEvent<S, N> for LoggerFormatter
where
    S: Collect + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // If using the new format, use the default formatters.
        if NEW_FORMAT.load(SeqCst) {
            return tracing_subscriber::fmt::format::Format::default()
                .with_level(SHOW_LEVEL.load(SeqCst))
                .with_file(SHOW_LOG_ORIGIN.load(SeqCst))
                .with_line_number(SHOW_LOG_ORIGIN.load(SeqCst))
                .format_event(ctx, writer, event);
        }

        // Format values from the event's's metadata:
        let metadata = event.metadata();

        let time = utils::time::LocalTime::now();
        let instance_id = logger::INSTANCE_ID
            .get()
            .map(String::as_str)
            .unwrap_or(logger::DEFAULT_INSTANCE_ID);
        let thread_id = std::thread::current()
            .name()
            .map(String::from)
            .unwrap_or(String::from("-"));

        // Write the time, instance ID and thread ID.
        write!(writer, "{time} [{instance_id}:{thread_id}")?;

        // Write the log level
        if SHOW_LEVEL.load(SeqCst) {
            write!(writer, ":{}", metadata.level())?;
        }

        // Write the log file and line.
        if SHOW_LOG_ORIGIN.load(SeqCst) {
            // Write the file
            write!(writer, ":{}", metadata.file().unwrap_or("unknown"))?;
            // Write the line
            if let Some(line) = metadata.line() {
                write!(writer, ":{line}")?;
            }
        }
        write!(writer, "] ")?;

        // Write fields on the event
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: u64 = 1024 * 1024;
    const ONE_TIME_BURST: u64 = 1024;
    const REFILL_TIME: u64 = 1000;

    #[test]
    fn test_rate_limiter_configs() {
        let rlconf = RateLimiterConfig {
            bandwidth: Some(TokenBucketConfig {
                size: SIZE,
                one_time_burst: Some(ONE_TIME_BURST),
                refill_time: REFILL_TIME,
            }),
            ops: Some(TokenBucketConfig {
                size: SIZE * 2,
                one_time_burst: None,
                refill_time: REFILL_TIME * 2,
            }),
        };
        let rl: RateLimiter = rlconf.try_into().unwrap();
        assert_eq!(rl.bandwidth().unwrap().capacity(), SIZE);
        assert_eq!(rl.bandwidth().unwrap().one_time_burst(), ONE_TIME_BURST);
        assert_eq!(rl.bandwidth().unwrap().refill_time_ms(), REFILL_TIME);
        assert_eq!(rl.ops().unwrap().capacity(), SIZE * 2);
        assert_eq!(rl.ops().unwrap().one_time_burst(), 0);
        assert_eq!(rl.ops().unwrap().refill_time_ms(), REFILL_TIME * 2);
    }

    #[test]
    fn test_generate_configs() {
        let bw_tb_cfg = TokenBucketConfig {
            size: SIZE,
            one_time_burst: Some(ONE_TIME_BURST),
            refill_time: REFILL_TIME,
        };
        let bw_tb = TokenBucket::new(SIZE, ONE_TIME_BURST, REFILL_TIME).unwrap();
        let generated_bw_tb_cfg = TokenBucketConfig::from(&bw_tb);
        assert_eq!(generated_bw_tb_cfg, bw_tb_cfg);

        let rl_conf = RateLimiterConfig {
            bandwidth: Some(bw_tb_cfg),
            ops: None,
        };
        let rl: RateLimiter = rl_conf.try_into().unwrap();
        let generated_rl_conf = RateLimiterConfig::from(&rl);
        assert_eq!(generated_rl_conf, rl_conf);
        assert_eq!(generated_rl_conf.into_option(), Some(rl_conf));
    }
}
