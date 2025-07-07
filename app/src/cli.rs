use clap::{
    Args, Parser, Subcommand, ValueEnum,
    builder::{Styles, styling::AnsiColor},
};

#[derive(Parser)]
#[command(version, about, long_about = None, styles = get_styles())]
pub struct Cli {
    /// Use verbose output (`-vv` very verbose output)
    #[arg(short, long = "verbose", action = clap::ArgAction::Count)]
    pub verbosity: u8,

    /// Number of parallel jobs to run (0 means auto-detect)
    #[arg(short, action = clap::ArgAction::Set, default_value = "0")]
    pub jobs: usize,

    #[command(subcommand)]
    pub subcommand: CliSubcommand,
}

#[derive(Subcommand)]
pub enum CliSubcommand {
    /// Show brief info about entities of current workspace
    Info(CommandInfoArgs),

    /// Search resources in the current workspace
    #[clap(visible_alias("q"))]
    Query(CommandQueryArgs),

    /// Explain how resources are transformed and imported into a project
    Explain(CommandExplainArgs),

    /// Download resources metadata from remote to cache
    #[clap(visible_alias("f"))]
    Fetch(CommandFetchArgs),

    /// Import resources from remotes to workspace files
    #[clap(visible_alias("i"))]
    Import(CommandImportArgs),

    /// Clean up application cache
    Clean(CommandCleanArgs),

    /// Add Figma personal token to secure storage
    Auth(CommandAuthArgs),
}

#[derive(Args, Debug)]
pub struct CommandQueryArgs {
    /// A label pattern describing the resources affected by a command
    pub pattern: Vec<String>,

    /// Customize command's output type
    #[arg(short, long, value_enum, default_value = "label")]
    pub output: QueryOutput,
}

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum QueryOutput {
    Label,
    Profile,
    Package,
    Tree,
}

#[derive(Args, Debug)]
pub struct CommandExplainArgs {
    /// A label pattern describing the resources affected by a command
    pub pattern: Vec<String>,
}

#[derive(Args, Debug)]
pub struct CommandInfoArgs {
    /// The name of the entity whose information should be output
    pub entity: InfoEntity,
}

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum InfoEntity {
    Workspace,
    Package,
}

#[derive(Args, Debug)]
pub struct CommandFetchArgs {
    /// A label pattern describing the resources affected by a command
    pub pattern: Vec<String>,
}

#[derive(Args, Debug)]
pub struct CommandImportArgs {
    /// A label pattern describing the resources affected by a command
    pub pattern: Vec<String>,

    /// Run fetch even if already have cached remote metadata
    #[arg(long)]
    pub refetch: bool,
}

#[derive(Args, Debug)]
pub struct CommandCleanArgs {
    /// Remove all metadata about remotes and all downloaded images
    #[arg(long)]
    pub all: bool,
}

#[derive(Args, Debug)]
pub struct CommandAuthArgs {
    /// Delete token from keychain
    #[arg(short, long)]
    pub delete: bool,
}

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().bold())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Cyan.on_default().bold())
        .placeholder(AnsiColor::Cyan.on_default())
}
