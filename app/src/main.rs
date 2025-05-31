use std::process::ExitCode;

use clap::Parser;
use cli::{
    Cli, CliSubcommand, CommandAQueryArgs, CommandCleanArgs, CommandFetchArgs, CommandImportArgs,
    CommandInfoArgs, CommandQueryArgs,
};
use command_aquery::FeatureAQueryOptions;
use command_clean::FeatureCleanOptions;
use command_fetch::FeatureFetchOptions;
use command_import::FeatureImportOptions;
use command_info::FeatureInfoOptions;
use command_query::FeatureQueryOptions;

mod cli;
mod error;
mod logging;
use error::*;
use log::Log;
use logging::{LOGGER, init_log_impl};

pub fn main() -> ExitCode {
    match run_app() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            LOGGER.wait().flush();
            handle_error(err);
            ExitCode::FAILURE
        }
    }
}

fn run_app() -> Result<()> {
    let cli = Cli::parse();
    init_log_impl(cli.verbosity, cli.quiet);

    match cli.subcommand {
        CliSubcommand::Info(CommandInfoArgs { entity }) => {
            command_info::info(FeatureInfoOptions {
                entity: match entity {
                    cli::InfoEntity::Workspace => command_info::InfoEntity::Workspace,
                    cli::InfoEntity::Package => command_info::InfoEntity::Package,
                },
            })?
        }

        CliSubcommand::Query(CommandQueryArgs { pattern, output }) => {
            command_query::query(FeatureQueryOptions {
                pattern,
                output: match output {
                    cli::QueryOutput::Label => command_query::QueryOutputType::Label,
                    cli::QueryOutput::Profile => command_query::QueryOutputType::Profile,
                    cli::QueryOutput::Package => command_query::QueryOutputType::Package,
                    cli::QueryOutput::Tree => command_query::QueryOutputType::Tree,
                },
            })?
        }

        CliSubcommand::AQuery(CommandAQueryArgs { pattern }) => {
            command_aquery::query(FeatureAQueryOptions { pattern })?
        }

        CliSubcommand::Fetch(CommandFetchArgs { pattern }) => {
            command_fetch::fetch(FeatureFetchOptions {
                pattern,
                concurrency: cli.jobs,
            })?
        }

        CliSubcommand::Import(CommandImportArgs { pattern, refetch }) => {
            command_import::import(FeatureImportOptions {
                pattern,
                refetch,
                concurrency: cli.jobs,
            })?
        }

        CliSubcommand::Clean(CommandCleanArgs { all }) => {
            command_clean::clean(FeatureCleanOptions { all })?
        }
    }
    Ok(())
}
