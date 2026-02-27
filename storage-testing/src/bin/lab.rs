use clap::{Parser, Subcommand};
use storage_testing::errors::Result;
use storage_testing::image_lab;
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

fn run_plan(plan: image_lab::Plan) -> Result<()> {
    for step in &plan.steps {
        println!("{}", step);
    }
    let _ = image_lab::execute_plan(&plan)?;
    Ok(())
}

fn attach_and_record(spec_name: &str, dry_run: bool) -> Result<()> {
    let plan = image_lab::plan_attach(spec_name, dry_run)?;
    let outcomes = image_lab::execute_plan(&plan)?;

    if !dry_run {
        let loops: Vec<String> = outcomes
            .iter()
            .filter(|result| result.executed)
            .map(|result| result.stdout.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect();
        image_lab::record_attach_state(spec_name, &loops)?;
    }

    Ok(())
}

fn mount_and_record(spec_name: &str, dry_run: bool) -> Result<()> {
    let plan = image_lab::plan_mount(spec_name, dry_run)?;
    let _ = image_lab::execute_plan(&plan)?;
    if !dry_run {
        let spec = spec::load_by_name(spec_name)?;
        let mount_points = spec
            .mounts
            .iter()
            .map(|mount| mount.mount_point.clone())
            .collect::<Vec<_>>();
        image_lab::record_mount_state(spec_name, &mount_points)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = LabCli::parse();
    match cli.command {
        LabCommand::Image { command } => match command {
            ImageCommand::Create { spec_name, dry_run } => {
                run_plan(image_lab::plan_create(&spec_name, dry_run)?)
            }
            ImageCommand::Prepare { spec_name, dry_run } => {
                run_plan(image_lab::plan_prepare(&spec_name, dry_run)?)
            }
            ImageCommand::Attach { spec_name, dry_run } => attach_and_record(&spec_name, dry_run),
            ImageCommand::Mount { spec_name, dry_run } => mount_and_record(&spec_name, dry_run),
            ImageCommand::Unmount { spec_name, dry_run } => {
                run_plan(image_lab::plan_unmount(&spec_name, dry_run)?)
            }
            ImageCommand::Detach { spec_name, dry_run } => {
                run_plan(image_lab::plan_detach(&spec_name, dry_run)?)
            }
            ImageCommand::Destroy { spec_name, dry_run } => {
                run_plan(image_lab::plan_destroy(&spec_name, dry_run)?)
            }
        },
        LabCommand::Cleanup {
            spec_name,
            all,
            dry_run,
        } => {
            if all {
                run_plan(image_lab::plan_cleanup_all(dry_run)?)
            } else {
                let spec = spec_name.expect("spec name is required unless --all is used");
                run_plan(image_lab::plan_cleanup(&spec, dry_run)?)
            }
        }
    }
}
