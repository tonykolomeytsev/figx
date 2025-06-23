use crate::{Dashboard, INSTANCE, render_progress_bar};
use crossterm::{
    cursor::MoveToColumn,
    queue,
    style::{Print, Stylize},
    terminal::{Clear, ClearType},
};
use log::{Log, max_level, set_logger};
use std::io::{Write, stderr};

impl Log for Dashboard {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut stderr = stderr().lock();
        let _ = match record.target().as_ref() {
            "" => Ok(()),
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

pub fn init_log_impl(verbosity: u8, _quiet: bool) {
    set_logger(&*INSTANCE).unwrap();

    // Устанавливаем уровень логгирования в зависимости от verbosity
    log::set_max_level(match verbosity {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    });
}
