use owo_colors::OwoColorize;

pub fn init_log_impl(verbosity: u8) {
    fern::Dispatch::new()
        .format(move |out, msg, record| match record.level() {
            log::Level::Info => match (verbosity, record.target()) {
                (0, "Downloading") | (0, "Converting") | (0, "Writing") => (),
                _ => out.finish(format_args!(
                    "{level: >12} {msg}",
                    level = record.target().green().bold(),
                )),
            },
            log::Level::Warn => out.finish(format_args!(
                "{level}: {msg}",
                level = "warning".yellow().bold(),
            )),
            log::Level::Error => {
                out.finish(format_args!("{level}: {msg}", level = "error".red().bold(),))
            }
            log::Level::Debug => out.finish(format_args!(
                "{level}: [{target}] {msg}",
                level = "debug".bright_black().bold(),
                target = record.metadata().target(),
            )),
            log::Level::Trace => out.finish(format_args!(
                "{level}: [{target}] {msg}",
                level = "trace".magenta().bold(),
                target = record.metadata().target(),
            )),
        })
        .chain(
            fern::Dispatch::new()
                .level(match verbosity {
                    0 | 1 => log::LevelFilter::Info,
                    2 => log::LevelFilter::Debug,
                    _ => log::LevelFilter::Trace,
                })
                // accept info messages from the current crate too
                .level_for("ureq", log::LevelFilter::Off)
                .level_for("ureq_proto", log::LevelFilter::Off)
                .level_for("rustls", log::LevelFilter::Off)
                .level_for("ignore", log::LevelFilter::Off)
                .chain(std::io::stderr()),
        )
        .apply()
        .unwrap();
}
