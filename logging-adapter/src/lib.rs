// Basic logging adapter implementation
use log::{debug, error, info, trace, warn};

// Log levels enum matching expected WASI interface
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

// Context structure matching expected WASI interface
pub struct Context {
    pub span_id: Option<u64>,
}

// The main logging adapter structure
pub struct LoggingAdapter;

impl LoggingAdapter {
    // Main logging function
    pub fn log(level: Level, context: Context, message: String) {
        match level {
            Level::Trace => trace!("{}", message),
            Level::Debug => debug!("{}", message),
            Level::Info => info!("{}", message),
            Level::Warn => warn!("{}", message),
            Level::Error => error!("{}", message),
        }
    }

    // Convenience methods
    pub fn trace(context: Context, message: String) {
        trace!("{}", message);
    }

    pub fn debug(context: Context, message: String) {
        debug!("{}", message);
    }

    pub fn info(context: Context, message: String) {
        info!("{}", message);
    }

    pub fn warn(context: Context, message: String) {
        warn!("{}", message);
    }

    pub fn error(context: Context, message: String) {
        error!("{}", message);
    }
}
