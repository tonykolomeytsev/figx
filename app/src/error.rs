use crossterm::style::Stylize;
use derive_more::From;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(From)]
pub enum Error {
    #[from]
    Info(command_info::Error),

    #[from]
    Query(command_query::Error),

    #[from]
    EQuery(command_aquery::Error),

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
        IO(err) => eprintln!(
            "{err_label} unable to access config file: {err}",
            err_label = "error:".red().bold(),
        ),
    }
}

fn handle_cmd_equery_error(err: command_aquery::Error) {
    use command_aquery::Error::*;
    match err {
        Pattern(err) => handle_pattern_error(err),
        Workspace(err) => handle_phase_loading_error(err),
        Analysis(err) => handle_evaluation_error(err),
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
        IO(err) => eprintln!(
            "{err_label} unable to delete cache directory: {err}",
            err_label = "error:".red().bold(),
        ),
        Evaluation(err) => handle_evaluation_error(err),
    }
}

fn handle_pattern_error(err: lib_label::PatternError) {
    eprintln!(
        "{err_label} entered pattern is incorrect:",
        err_label = "error:".red().bold(),
    );

    use lib_label::PatternError::*;
    match err {
        BadPackage(pattern, package) => {
            eprintln!(
                // --- pattern
                "  {arrow} '{full_pattern}'\n\
                {s: <7}{underline} {help}\n\n\
                {tip_label} valid package patterns are: '//foo/bar' 'buz/...', '//...', or even empty\n",
                // --- args
                arrow = "-->".cyan(),
                full_pattern = pattern,
                s = "",
                underline = "^".repeat(package.len()).yellow().bold(),
                help = "help: package pattern contains invalid characters"
                    .yellow()
                    .bold(),
                tip_label = "  tip:".green(),
            )
        }
        BadTarget(pattern, target) => {
            let pos = pattern.find(':').unwrap_or_default();
            eprintln!(
                // --- pattern
                "  {arrow} '{full_pattern}'\n\
                {s}{underline} {help}\n\n\
                {tip_label} valid target pattern are: '*', '*-16px', 'ic_*_24', '*Icon', 'app-logo'\n",
                // --- args
                arrow = "-->".cyan(),
                full_pattern = pattern,
                s = " ".repeat(pos + 8),
                underline = "^".repeat(target.len()).yellow().bold(),
                help = if target.is_empty() {
                    "help: target pattern mustn't be empty"
                } else {
                    "help: target pattern contains invalid characters"
                }
                .yellow()
                .bold(),
                tip_label = "  tip:".green(),
            )
        }
    }
}

fn handle_phase_loading_error(err: phase_loading::Error) {
    use phase_loading::Error::*;
    match err {
        Internal(str) => {
            eprintln!(
                "{err_label} internal error: {description}",
                err_label = "error:".red().bold(),
                description = str,
            );
        }
        InitNotInWorkspace => {
            eprintln!(
                "{err_label} current working directory is not part of the figmagic workspace\n\n\
                {tip_label} workspace is the root directory of the project, which contains the file '.figmagic.toml'.\n",
                err_label = "error:".red().bold(),
                tip_label = "  tip:".green(),
            );
        }
        InitInaccessibleCurrentWorkDir => {
            eprintln!(
                "{err_label} unable to access current working directory\n\n\
                {tip_label} there may be some file access rights issues\n",
                err_label = "error:".red().bold(),
                tip_label = "  tip:".green(),
            );
        }
        WorkspaceRead(err) => {
            eprintln!(
                "{err_label} unable to read workspace file '.figmagic.toml':\n\n{err}\n",
                err_label = "error:".red().bold(),
            );
        }
        WorkspaceParse(err) => {
            eprintln!(
                "{err_label} failed to parse workspace file '.figmagic.toml':\n\n{err}\n",
                err_label = "error:".red().bold(),
            );
        }
        WorkspaceNoRemotes => {
            eprintln!(
                "{err_label} no remotes specified in workspace file '.figmagic.toml'\n\n\
                {tip_label} at least one remote must be specified, e.g. '[remotes.figma]'\n",
                err_label = "error:".red().bold(),
                tip_label = "  tip:".green(),
            );
        }
        WorkspaceRemoteNoAccessToken(id) => {
            eprintln!(
                "{err_label} remote '{id}' has no access token specified \n\n\
                {s: <7}{access_token} = \"some-token\"\n\
                {tabs} {underline}\n\n\
                {tabs} ... or ...\n\n\
                {s: <7}{access_token} = \"FIGMA_ACCESS_TOKEN\"\n\
                {tabs} {underline}\n",
                err_label = "error:".red().bold(),
                s = "",
                access_token = "access_token.env".green(),
                tabs = " ".repeat(6),
                underline = "+".repeat(16).green().bold(),
            );
        }
        WorkspaceMoreThanOneDefaultRemotes => {
            eprintln!(
                "{err_label} the default remote can only be one\n",
                err_label = "error:".red().bold(),
            );
        }
        WorkspaceAtLeastOneDefaultRemote => {
            eprintln!(
                "{err_label} at least one remote must be selected by default\n\n\
                {s: <7}{default} = true\n\
                {tabs} {underline}\n",
                err_label = "error:".red().bold(),
                s = "",
                default = "default".green(),
                tabs = " ".repeat(6),
                underline = "+".repeat(7).green().bold(),
            );
        }
        WorkspaceRemoteWithEmptyNodeId => {
            eprintln!(
                "{err_label} remote has empty container_node_id list",
                err_label = "error:".red().bold(),
            );
        }
        WorkspaceInvalidProfileToExtend(from, to) => {
            eprintln!(
                "{err_label} profile {from} cannot be extended with {to}",
                err_label = "error:".red().bold(),
            );
        }
        FigTraversing(err) => {
            eprintln!(
                "{err_label} internal error: {description}",
                err_label = "error:".red().bold(),
                description = err,
            );
        }
        FigRead(err) => {
            eprintln!(
                "{err_label} unable to read fig file '.fig.toml':\n\n{err}\n",
                err_label = "error:".red().bold(),
            );
        }
        FigParse(err) => {
            eprintln!(
                "{err_label} failed to parse fig file '.fig.toml':\n\n{err}\n",
                err_label = "error:".red().bold(),
            );
        }
        FigInvalidResourceName(err) => handle_name_parsing_error(err),
        FigInvalidPackage(err) => handle_package_parsing_error(err),
        FigInvalidProfileName(err) => {
            eprintln!(
                "{err_label} invalid profile name '{err}'\n",
                err_label = "error:".red().bold(),
            );
        }
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
            {tip_label} if the problem persists, run 'figmagic clean'\n",
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
