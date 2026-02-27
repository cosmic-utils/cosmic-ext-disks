use clap::{Parser, Subcommand};
use storage_testing::errors::Result;
use storage_testing::lab::orchestrator;
use storage_testing::lab::orchestrator::ExecuteOptions;
use storage_testing::spec;

#[derive(Debug, Parser)]
#[command(name = "lab")]
#[command(about = "Host image lab lifecycle commands for storage testing")]
struct LabCli {
    #[command(subcommand)]
    command: LabCommand,
}

#[derive(Debug, Subcommand)]
enum LabCommand {
    Image {
        #[command(subcommand)]
        command: ImageCommand,
    },
    Cleanup {
        spec_name: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ImageCommand {
    Create {
        spec_name: String,
        #[arg(long)]
        dry_run: bool,
    },
    Prepare {
        spec_name: String,
        #[arg(long)]
        dry_run: bool,
    },
    Attach {
        spec_name: String,
        #[arg(long)]
        dry_run: bool,
    },
    Mount {
        spec_name: String,
        #[arg(long)]
        dry_run: bool,
    },
    Unmount {
        spec_name: String,
        #[arg(long)]
        dry_run: bool,
    },
    Detach {
        spec_name: String,
        #[arg(long)]
        dry_run: bool,
    },
    Destroy {
        spec_name: String,
        #[arg(long)]
        dry_run: bool,
    },
}

fn print_outcomes(outcomes: Vec<storage_testing::cmd::CommandOutcome>) {
    for outcome in outcomes {
        println!("{}", outcome.command);
    }
}

fn main() -> Result<()> {
    let cli = LabCli::parse();
    match cli.command {
        LabCommand::Image { command } => match command {
            ImageCommand::Create { spec_name, dry_run } => {
                print_outcomes(orchestrator::create(
                    &spec_name,
                    ExecuteOptions { dry_run },
                )?);
                Ok(())
            }
            ImageCommand::Prepare { spec_name, dry_run } => {
                print_outcomes(orchestrator::prepare(
                    &spec_name,
                    ExecuteOptions { dry_run },
                )?);
                Ok(())
            }
            ImageCommand::Attach { spec_name, dry_run } => {
                print_outcomes(orchestrator::attach(
                    &spec_name,
                    ExecuteOptions { dry_run },
                )?);
                Ok(())
            }
            ImageCommand::Mount { spec_name, dry_run } => {
                print_outcomes(orchestrator::mount(
                    &spec_name,
                    ExecuteOptions { dry_run },
                )?);
                Ok(())
            }
            ImageCommand::Unmount { spec_name, dry_run } => {
                print_outcomes(orchestrator::unmount(
                    &spec_name,
                    ExecuteOptions { dry_run },
                )?);
                Ok(())
            }
            ImageCommand::Detach { spec_name, dry_run } => {
                print_outcomes(orchestrator::detach(
                    &spec_name,
                    ExecuteOptions { dry_run },
                )?);
                Ok(())
            }
            ImageCommand::Destroy { spec_name, dry_run } => {
                print_outcomes(orchestrator::destroy(
                    &spec_name,
                    ExecuteOptions { dry_run },
                )?);
                Ok(())
            }
        },
        LabCommand::Cleanup {
            spec_name,
            all,
            dry_run,
        } => {
            if all {
                print_outcomes(orchestrator::cleanup_all(ExecuteOptions { dry_run })?);
                Ok(())
            } else {
                let spec = spec_name.expect("spec name is required unless --all is used");
                let _ = spec::load_by_name(&spec)?;
                print_outcomes(orchestrator::cleanup(
                    &spec,
                    ExecuteOptions { dry_run },
                )?);
                Ok(())
            }
        }
    }
}
