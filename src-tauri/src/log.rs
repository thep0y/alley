use std::{env, str::FromStr, sync::Mutex};

use time::{format_description::BorrowedFormatItem, macros::format_description};
use tracing::Level;
use tracing_subscriber::{
    fmt::{time::LocalTime, writer::BoxMakeWriter},
    EnvFilter,
};

/// Get default log level
fn default_log_level() -> Level {
    if cfg!(any(debug_assertions, mobile)) {
        Level::TRACE
    } else {
        Level::WARN
    }
}

/// Get log level from environment variable
fn get_log_level() -> Level {
    env::var("FLUXY_LOG")
        .ok()
        .and_then(|level| Level::from_str(&level).ok())
        .unwrap_or_else(default_log_level)
}

/// Get time format based on build configuration
fn get_time_format<'a>() -> &'a [BorrowedFormatItem<'a>] {
    if cfg!(debug_assertions) {
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]")
    } else {
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]")
    }
}

/// Setup logging with given configuration
pub fn setup_logging() {
    let timer = LocalTime::new(get_time_format());

    let writer = if cfg!(any(debug_assertions, mobile)) {
        BoxMakeWriter::new(Mutex::new(std::io::stderr()))
    } else {
        use crate::lazy::FLUXY_CONFIG_DIR;
        use std::fs::File;

        let log_file = File::create(FLUXY_CONFIG_DIR.join("fluxy.log"))
            .expect("Failed to create the log file");
        BoxMakeWriter::new(Mutex::new(log_file))
    };

    let level = get_log_level();

    let builder = tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(EnvFilter::new(format!("fluxy_lib={level}")))
        .with_target(false)
        .with_timer(timer)
        .with_writer(writer);

    if cfg!(any(debug_assertions, mobile)) {
        builder.with_ansi(true).init();
    } else {
        builder.json().init();
    }
}
