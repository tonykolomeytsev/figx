use crossterm::{
    cursor::MoveToColumn,
    queue,
    style::{Print, Stylize},
    terminal::{Clear, ClearType},
};
use lib_progress_bar::{get_progress_bar_display, is_progress_bar_visible};
use log::{max_level, set_logger};
use std::{
    io::{Write, stderr},
    sync::LazyLock,
};

pub static LOGGER: LazyLock<Logger> = LazyLock::new(|| Logger);

/// A simple logger.
pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        if should_skip_log(record) {
            return;
        }

        let level = record.level();
        let target = record.target();
        let msg = record.args();

        let mut stdout = stderr().lock();
        let label = match level {
            log::Level::Trace => "trace:".to_owned().bold().magenta(),
            log::Level::Debug => "debug:".to_owned().bold().grey(),
            log::Level::Warn => "warning:".to_owned().bold().yellow(),
            log::Level::Error => "error:".to_owned().bold().red(),
            log::Level::Info => format!("{target: >12}").bold().green(),
        };
        let _ = queue!(
            stdout,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            Print(format!("{label} {msg}\n")),
        );
        if is_progress_bar_visible() {
            let _ = queue!(
                stdout,
                Print(format!(
                    "{} {}",
                    "   Executing".bold().cyan(),
                    get_progress_bar_display()
                ))
            );
        }
        let _ = stdout.flush();
    }

    fn flush(&self) {
        let mut stdout = stderr().lock();
        let _ = queue!(
            stdout,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
        );
    }
}

fn should_skip_log(record: &log::Record) -> bool {
    let level = record.metadata().level();
    let target = record.target();

    if target.starts_with("ureq") && level != log::LevelFilter::Error {
        return true;
    }
    if target.starts_with("ureq_proto") && level != log::LevelFilter::Error {
        return true;
    }
    if target.starts_with("rustls") && level != log::LevelFilter::Error {
        return true;
    }
    if target.starts_with("ignore") && level != log::LevelFilter::Error {
        return true;
    }
    false
}

pub fn init_log_impl(verbosity: u8) {
    set_logger(&*LOGGER).unwrap();

    // Устанавливаем уровень логгирования в зависимости от verbosity
    log::set_max_level(match verbosity {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    });
}
