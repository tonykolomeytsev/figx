use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFile,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use crossterm::style::Stylize;
use derive_more::From;
use std::{fmt::Display, ops::Range, path::Path};
use toml_span::ErrorKind;
use unindent::unindent;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(From)]
pub enum Error {
    #[from]
    Info(command_info::Error),

    #[from]
    Query(command_query::Error),

    #[from]
    EQuery(command_explain::Error),

    #[from]
    Fetch(command_fetch::Error),

    #[from]
    Import(command_import::Error),

    #[from]
    Clean(command_clean::Error),
}

pub fn handle_error(err: Error) {
    use Error::*;
    match err {
        Info(err) => handle_cmd_info_error(err),
        Query(err) => handle_cmd_query_error(err),
        EQuery(err) => handle_cmd_equery_error(err),
        Fetch(err) => handle_cmd_fetch_error(err),
        Import(err) => handle_cmd_import_error(err),
        Clean(err) => handle_cmd_clean_error(err),
    }
}

fn handle_cmd_info_error(err: command_info::Error) {
    use command_info::Error::*;
    match err {
        InitError(err) => handle_phase_loading_error(err),
    }
}

fn handle_cmd_query_error(err: command_query::Error) {
    use command_query::Error::*;
    match err {
        PatternError(err) => handle_pattern_error(err),
        WorkspaceError(err) => handle_phase_loading_error(err),
        IO(err) => cli_input_error(CliInputDiagnostics {
            message: &format!("unable to access config file: {err}"),
            labels: &[],
        }),
    }
}

fn handle_cmd_equery_error(err: command_explain::Error) {
    use command_explain::Error::*;
    match err {
        Pattern(err) => handle_pattern_error(err),
        Workspace(err) => handle_phase_loading_error(err),
    }
}

fn handle_cmd_fetch_error(err: command_fetch::Error) {
    use command_fetch::Error::*;
    match err {
        Pattern(err) => handle_pattern_error(err),
        Workspace(err) => handle_phase_loading_error(err),
        Evaluation(err) => handle_evaluation_error(err),
    }
}

fn handle_cmd_import_error(err: command_import::Error) {
    use command_import::Error::*;
    match err {
        Pattern(err) => handle_pattern_error(err),
        Workspace(err) => handle_phase_loading_error(err),
        Evaluation(err) => handle_evaluation_error(err),
    }
}

fn handle_cmd_clean_error(err: command_clean::Error) {
    use command_clean::Error::*;
    match err {
        WorkspaceError(err) => handle_phase_loading_error(err),
        IO(err) => cli_input_error(CliInputDiagnostics {
            message: &format!("unable to delete cache directory: {err}"),
            labels: &[],
        }),
        Evaluation(err) => handle_evaluation_error(err),
    }
}

fn handle_pattern_error(err: lib_label::PatternError) {
    use lib_label::PatternError::*;
    match err {
        BadPackage(pattern, package) => cli_input_error(CliInputDiagnostics {
            message: &format!("entered pattern is incorrect: `{pattern}`"),
            labels: &[
                CliInputLabel::Tip(&unindent::unindent(
                    "
                        valid package patterns are: 
                        - `//foo/bar`
                        - `buz/...`
                        - `//...`
                        - or even empty
                    ",
                )),
                CliInputLabel::YellowHelp(
                    &pattern,
                    0..package.len(),
                    "package pattern contains invalid characters",
                ),
            ],
        }),
        BadTarget(pattern, target) => {
            let pos = pattern.find(':').unwrap_or_default();
            cli_input_error(CliInputDiagnostics {
                message: &format!("entered pattern is incorrect: `{pattern}`"),
                labels: &[
                    CliInputLabel::Tip(&unindent::unindent(
                        "
                        valid target patterns are: 
                        - *
                        - *-16,
                        - ic_*_24
                        - *Icon
                        - StarOutline24,
                    ",
                    )),
                    CliInputLabel::YellowHelp(
                        &pattern,
                        pos..target.len(),
                        if target.is_empty() {
                            "^ target pattern mustn't be empty"
                        } else {
                            "target pattern contains invalid characters"
                        },
                    ),
                ],
            })
        }
    }
}

fn handle_phase_loading_error(err: phase_loading::Error) {
    use phase_loading::Error::*;
    match err {
        Internal(str) => cli_input_error(CliInputDiagnostics {
            message: &format!("[internal] {str}"),
            labels: &[],
        }),
        InitNotInWorkspace => cli_input_error(CliInputDiagnostics {
            message: "current working directory is not part of the FigX workspace",
            labels: &[CliInputLabel::Tip(&unindent::unindent(
                "
                    A `workspace` is the root directory of a project/repository that contains 
                    the marker file `.figtree.toml` and all its child directories.
                ",
            ))],
        }),
        InitInaccessibleCurrentWorkDir => cli_input_error(CliInputDiagnostics {
            message: "unable to access current working directory",
            labels: &[CliInputLabel::Tip(
                "there may be some file access rights issues",
            )],
        }),
        WorkspaceRead(err) => cli_input_error(CliInputDiagnostics {
            message: &format!("unable to read workspace file '.figtree.toml': {err}"),
            labels: &[],
        }),
        WorkspaceParse(err, path) => handle_toml_parsing_error(
            err,
            &path,
            "failed to parse workspace file `.figtree.toml`",
        ),
        WorkspaceRemoteNoAccessToken(id, path, span) => {
            let file = create_simple_file(&path);
            let diagnostic = Diagnostic::error()
                .with_message(format!("remote `{id}` has no access token specified"))
                .with_note(unindent(
                    "
                        consider using `access_token.env = \"ENV_WITH_TOKEN\"`
                        or specify FIGMA_PERSONAL_TOKEN in your environment
                    ",
                ))
                .with_label(Label::primary((), span));
            print_codespan_diag(diagnostic, &file);
        }
        FigTraversing(err) => cli_input_error(CliInputDiagnostics {
            message: &format!("[internal] fig-files traversing: {err}"),
            labels: &[CliInputLabel::Tip(
                "there may be some file access rights issues",
            )],
        }),
        FigRead(err) => cli_input_error(CliInputDiagnostics {
            message: &format!("unable to read fig-file: {err}"),
            labels: &[CliInputLabel::Tip(
                "there may be some file access rights issues",
            )],
        }),
        FigParse(err, path) => {
            handle_toml_parsing_error(err, &path, "failed to parse fig-file `.fig.toml`")
        }
        FigInvalidResourceName(err) => handle_name_parsing_error(err),
        FigInvalidPackage(err) => handle_package_parsing_error(err),
        FigInvalidRemoteName(remote) => {
            eprintln!(
                "{err_label} invalid remote name '{name}'\n",
                err_label = "error:".red().bold(),
                name = remote.yellow(),
            );
        }
    }
}

fn handle_name_parsing_error(err: lib_label::NameParsingError) {
    eprintln!(
        "{err_label} invalid resource name: '{res_name}'\n\n\
        {tip_label} valid resource name contains only numbers, latin letters, underlines and dashes\n",
        err_label = "error:".red().bold(),
        res_name = err.0.yellow(),
        tip_label = "  tip:".green(),
    );
}

fn handle_package_parsing_error(err: lib_label::PackageParsingError) {
    eprintln!(
        "{err_label} invalid package: '{pkg_name}'\n\n\
        {tip_label} package looks kinda sus...\n",
        err_label = "error:".red().bold(),
        pkg_name = err.0.yellow(),
        tip_label = "  tip:".green(),
    );
}

fn handle_evaluation_error(err: phase_evaluation::Error) {
    use phase_evaluation::Error::*;
    match err {
        IO(err) => eprintln!(
            "{err_label} io error: {err}",
            err_label = "error:".red().bold(),
        ),
        Cache(err) => eprintln!(
            "{err_label} cache error: '{err}'\n\n\
            {tip_label} if the problem persists, run 'figx clean'\n",
            err_label = "error:".red().bold(),
            tip_label = "  tip:".green(),
        ),
        WebpCreate => eprintln!(
            "{err_label} while converting PNG to WEBP\n\n\
            {tip_label} only RGB8 and ARGB8 profiles are supported\n",
            err_label = "error:".red().bold(),
            tip_label = "  tip:".green(),
        ),
        ImageDecode(err) => eprintln!(
            "{err_label} while decoding image from Figma: {err}",
            err_label = "error:".red().bold(),
        ),
        FigmaApiNetwork(err) => {
            use ureq::Error::*;
            match err.0 {
                StatusCode(code) if code == 403 => eprintln!(
                    "{err_label} while requesting Figma API: invalid access token",
                    err_label = "error:".red().bold(),
                ),
                StatusCode(code) if code == 429 => eprintln!(
                    "{err_label} too many requests to Figma API",
                    err_label = "error:".red().bold(),
                ),
                err => eprintln!(
                    "{err_label} while requesting Figma API: {err}",
                    err_label = "error:".red().bold(),
                ),
            }
        }
        ExportImage(err) => eprintln!(
            "{err_label} while exporting image: {err}",
            err_label = "error:".red().bold(),
        ),
        FindNode { node_name } => eprintln!(
            "{err_label} cannot find node with name '{node_name}'",
            err_label = "error:".red().bold(),
        ),
        ActionSingleInputAbsent => eprintln!(
            "{err_label} internal: action input is absent",
            err_label = "error:".red().bold(),
        ),
        ActionTaggedInputAbsent => eprintln!(
            "{err_label} internal: tagged action input is absent",
            err_label = "error:".red().bold(),
        ),
        SvgToCompose(err) => {
            eprintln!("{err_label} {err:?}", err_label = "error:".red().bold());
        }
        Interrupted(err) => {
            eprintln!("{err_label} {err}", err_label = "error:".red().bold());
        }
    }
}

struct CliInputDiagnostics<'a> {
    message: &'a str,
    labels: &'a [CliInputLabel<'a>],
}

#[allow(unused)]
enum CliInputLabel<'a> {
    Suggestion(&'a str),
    YellowHelp(&'a str, Range<usize>, &'a str),
    Tip(&'a str),
}

fn cli_input_error(args: CliInputDiagnostics) {
    let err_label = "error:".red().bold();
    let tip_label = "tip:".green();
    let CliInputDiagnostics { message, labels } = args;
    eprintln!("{err_label} {message}");
    for label in labels {
        use CliInputLabel::*;
        match label {
            Suggestion(s) => {
                eprintln!("\n       {}", s.green());
                eprintln!("       {}", "+".repeat(s.len()).green())
            }
            YellowHelp(s1, rng, desc) => {
                let help_label = "help:".bold().yellow();
                let desc = desc.bold().yellow();
                eprintln!("\n {help_label} {}", s1.bold().white());
                eprintln!(
                    "       {}{} {desc}",
                    " ".repeat(rng.start),
                    "^".repeat(rng.end).yellow().bold(),
                );
            }
            Tip(s) => {
                for (n, line) in s.lines().enumerate() {
                    if n == 0 {
                        eprintln!("\n  {tip_label} {line}")
                    } else {
                        eprintln!("       {line}")
                    }
                }
            }
        }
    }
}

fn create_simple_file(path: &Path) -> SimpleFile<String, String> {
    SimpleFile::new(
        path.display().to_string(),
        std::fs::read_to_string(path).expect("file is available right after parsing"),
    )
}

fn print_codespan_diag<A: Display + Clone, B: AsRef<str>>(
    diagnostic: Diagnostic<()>,
    file: &SimpleFile<A, B>,
) {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = term::Config::default();
    let _ = term::emit(&mut writer.lock(), &config, file, &diagnostic);
}

fn handle_toml_parsing_error(err: toml_span::DeserError, path: &Path, msg: &str) {
    let file = create_simple_file(&path);
    for err in err.errors {
        let mut diagnostic = Diagnostic::error().with_message(msg);

        match err {
            toml_span::Error {
                kind: ErrorKind::UnexpectedKeys { keys, expected },
                ..
            } => {
                for (key, span) in keys.into_iter() {
                    diagnostic = diagnostic
                        .with_label(
                            Label::primary((), span)
                                .with_message(format!("unexpected key '{key}'")),
                        )
                        .with_note(format!("possible keys are: {}", expected.join(", ")));
                }
            }
            err => {
                diagnostic = diagnostic
                    .with_label(Label::primary((), err.span).with_message(err.to_string()))
            }
        }
        print_codespan_diag(diagnostic, &file);
    }
}
