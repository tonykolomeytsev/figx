use env_logger::fmt::Formatter;
use log::Record;
use owo_colors::OwoColorize;
use std::io::Write;

pub fn init_log_impl(verbosity: u8) {
    env_logger::builder()
        .format(figmagic_format)
        .filter_level(match verbosity {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .init();
}

fn figmagic_format(buf: &mut Formatter, record: &Record<'_>) -> std::io::Result<()> {
    match record.level() {
        log::Level::Info => writeln!(
            buf,
            "{level}: {msg}",
            level = "info".blue().bold(),
            msg = record.args(),
        ),
        log::Level::Warn => writeln!(
            buf,
            "{level}: {msg}",
            level = "warning".yellow().bold(),
            msg = record.args(),
        ),
        log::Level::Error => writeln!(
            buf,
            "{level}: {msg}",
            level = "error".red().bold(),
            msg = record.args(),
        ),
        log::Level::Debug => writeln!(
            buf,
            "{level}: [{target}] {msg}",
            level = "debug".green().bold(),
            target = record.metadata().target(),
            msg = record.args(),
        ),
        log::Level::Trace => writeln!(
            buf,
            "{level}: [{target}] {msg}",
            level = "trace".magenta().bold(),
            target = record.metadata().target(),
            msg = record.args(),
        ),
    }
}
