use clap::{
    Args, Parser, Subcommand, ValueEnum,
    builder::{Styles, styling::AnsiColor},
};

#[derive(Parser)]
#[command(version, about, long_about = None, styles = get_styles())]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long = "verbose", action = clap::ArgAction::Count)]
    pub verbosity: u8,

    #[command(subcommand)]
    pub subcommand: CliSubcommand,
}

#[derive(Subcommand)]
pub enum CliSubcommand {
    /// Show brief info about entities of current workspace
    Info(CommandInfoArgs),

    /// Search resources in the current workspace
    Query(CommandQueryArgs),

    /// Analyze the action graph of resources in the current workspace
    #[clap(name = "aquery")]
    AQuery(CommandAQueryArgs),

    /// Download resources metadata from remote to cache
    Fetch(CommandFetchArgs),

    /// Import resources from remotes to workspace files
    Import(CommandImportArgs),

    /// Clean up application cache
    Clean,
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
pub struct CommandAQueryArgs {
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

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().bold())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Cyan.on_default().bold())
        .placeholder(AnsiColor::Cyan.on_default())
}
