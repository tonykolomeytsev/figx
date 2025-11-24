use crate::{Dashboard, INSTANCE, render_progress_bar};
use crossterm::{
    cursor::MoveToColumn,
    queue,
    style::{Print, Stylize},
    terminal::{Clear, ClearType},
};
use log::{Level, Log, Record, info, max_level, set_logger};
use std::io::{Write, stderr};

impl Log for Dashboard {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.target().starts_with('@') || metadata.level() <= max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        if should_skip(&record) {
            return;
        }

        let mut stderr = stderr().lock();
        let _ = match record.target().as_ref() {
            "@" => Ok(()),
            target if target.starts_with("@") => {
                queue!(
                    stderr,
                    MoveToColumn(0),
                    Print(format!(
                        "{} {}",
                        format!("{: >12}", target.trim_start_matches("@"))
                            .bold()
                            .green(),
                        record.args(),
                    )),
                    Clear(ClearType::UntilNewLine),
                    Print('\n'),
                )
            }
            target => {
                use log::Level::*;
                let label = match record.level() {
                    Trace => "trace:".bold().magenta(),
                    Debug => "debug:".bold().grey(),
                    Warn => "warning:".bold().yellow(),
                    Error => "error:".bold().red(),
                    Info => "info:".bold().cyan(),
                };
                queue!(
                    stderr,
                    MoveToColumn(0),
                    Print(format!("{label} [{target}] {}", record.args())),
                    Clear(ClearType::UntilNewLine),
                    Print('\n'),
                )
            }
        };
        let _ = render_progress_bar(&mut INSTANCE.progress_bar.lock().unwrap());
        let _ = stderr.flush();
    }

    fn flush(&self) {
        let _ = stderr().flush();
    }
}

pub fn init_log_impl(verbosity: u8) {
    set_logger(&*INSTANCE).unwrap();

    // Устанавливаем уровень логгирования в зависимости от verbosity
    let running_on_ci = is_ci::uncached();
    let force_debug_logging = std::env::var("DEBUG")
        .or(std::env::var("ACTIONS_RUNNER_DEBUG"))
        .or(std::env::var("ACTIONS_STEP_DEBUG"))
        .is_ok();
    log::set_max_level(match (verbosity, running_on_ci, force_debug_logging) {
        (_, _, true) => log::LevelFilter::Debug,
        (0, true, _) | (1, true, _) => log::LevelFilter::Info,
        (0, _, _) => log::LevelFilter::Warn,
        (1, _, _) => log::LevelFilter::Info,
        (2, _, _) => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    });

    if running_on_ci && !force_debug_logging {
        info!(target: "Logger", "CI environment detected, set verbosity to INFO")
    }
    if force_debug_logging {
        info!(target: "Logger", "Debug logs were enabled via environment variables")
    }
}

fn should_skip(record: &Record) -> bool {
    match record.target() {
        t if t.starts_with("ureq") => match record.level() {
            Level::Error | Level::Warn | Level::Info => false,
            _ => true,
        },
        t if t.starts_with("globset") => true,
        t if t.starts_with("ignore") => true,
        t if t.starts_with("rustls") => true,
        _ => false,
    }
}

#[macro_export]
macro_rules! lifecycle {
    (target: $target:expr, $($arg:tt)+) => ({
        log::log!(target: $target, log::Level::Warn, $($arg)+)
    });
}
