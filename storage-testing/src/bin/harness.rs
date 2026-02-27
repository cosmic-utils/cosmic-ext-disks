use clap::{Parser, Subcommand};
use storage_testing::errors::Result;
use storage_testing::harness;

#[derive(Debug, Parser)]
#[command(name = "harness")]
#[command(about = "Containerized integration harness for storage testing")]
struct HarnessCli {
    #[command(subcommand)]
    command: HarnessCommand,
}

#[derive(Debug, Subcommand)]
enum HarnessCommand {
    Run {
        #[arg(long, default_value = "harness_smoke")]
        suite: String,
        #[arg(long, default_value = "auto")]
        runtime: String,
        #[arg(long)]
        keep: bool,
        #[arg(long)]
        dry_run: bool,
    },
    Shell {
        #[arg(long, default_value = "auto")]
        runtime: String,
    },
    Cleanup {
        #[arg(long, default_value = "auto")]
        runtime: String,
    },
}

fn print_steps(plan: &harness::Plan) {
    for step in &plan.steps {
        println!("{}", step);
    }
}

fn main() -> Result<()> {
    let cli = HarnessCli::parse();

    match cli.command {
        HarnessCommand::Run {
            suite,
            runtime,
            keep,
            dry_run,
        } => {
            let plan = harness::plan_run(&suite, &runtime, keep, dry_run)?;
            let artifact_dir = harness::init_artifacts()?;
            println!("artifacts: {}", artifact_dir.display());
            print_steps(&plan);
            let _ = harness::execute_plan(&plan)?;
            Ok(())
        }
        HarnessCommand::Shell { runtime } => {
            let plan = harness::plan_shell(&runtime)?;
            let _ = harness::execute_plan(&plan)?;
            print_steps(&plan);
            Ok(())
        }
        HarnessCommand::Cleanup { runtime } => {
            let plan = harness::plan_cleanup(&runtime)?;
            let _ = harness::execute_plan(&plan)?;
            print_steps(&plan);
            Ok(())
        }
    }
}
