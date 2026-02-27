use clap::{Parser, Subcommand};
use storage_testing::errors::{Result, TestingError};
use storage_testing::harness::orchestrator::{self, CaseStatus, RunConfig};
use storage_testing::lab::orchestrator as lab;
use storage_testing::lab::orchestrator::ExecuteOptions;

#[derive(Debug, Parser)]
#[command(name = "harness")]
#[command(about = "Host integration harness for storage testing")]
struct HarnessCli {
    #[command(subcommand)]
    command: HarnessCommand,
}

#[derive(Debug, Subcommand)]
enum HarnessCommand {
    Run {
        #[arg(long)]
        suite: Option<String>,
        #[arg(long)]
        test_id: Option<String>,
        #[arg(long, default_value_t = 1)]
        max_parallel_groups: usize,
        #[arg(long)]
        dry_run: bool,
    },
    Cleanup {
        #[arg(long)]
        dry_run: bool,
    },
}

fn require_root_if_needed(dry_run: bool) -> Result<()> {
    if dry_run {
        return Ok(());
    }

    let output = std::process::Command::new("id")
        .arg("-u")
        .output()
        .map_err(|_| TestingError::PrivilegeRequired)?;

    let uid = String::from_utf8_lossy(&output.stdout);
    if uid.trim() == "0" {
        Ok(())
    } else {
        Err(TestingError::PrivilegeRequired)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = HarnessCli::parse();

    match cli.command {
        HarnessCommand::Run {
            suite,
            test_id,
            max_parallel_groups,
            dry_run,
        } => {
            require_root_if_needed(dry_run)?;
            let summary = orchestrator::run_all(RunConfig {
                suite,
                test_id,
                max_parallel_groups,
                dry_run,
            })
            .await?;

            let mut failed = false;
            let mut passed_count = 0usize;
            let mut skipped_count = 0usize;
            let mut failed_count = 0usize;
            for group in &summary.groups {
                let mut group_pass = 0usize;
                let mut group_skip = 0usize;
                let mut group_fail = 0usize;
                if let Some(error) = &group.setup_error {
                    println!("group {} setup_error: {}", group.spec, error);
                    failed = true;
                }

                for test in &group.tests {
                    match &test.status {
                        CaseStatus::Passed => {
                            passed_count += 1;
                            group_pass += 1;
                        }
                        CaseStatus::Skipped(reason) => {
                            skipped_count += 1;
                            group_skip += 1;
                            let _ = reason;
                        }
                        CaseStatus::Failed(reason) => {
                            failed_count += 1;
                            group_fail += 1;
                            println!("group {} FAIL {} ({})", group.spec, test.id, reason);
                            failed = true;
                        }
                    }
                }

                if let Some(error) = &group.teardown_error {
                    println!("group {} teardown_error: {}", group.spec, error);
                    failed = true;
                }

                println!(
                    "group {} summary: PASS={} SKIP={} FAIL={}",
                    group.spec, group_pass, group_skip, group_fail
                );
            }

            println!(
                "summary: PASS={} SKIP={} FAIL={}",
                passed_count, skipped_count, failed_count
            );

            if failed {
                return Err(
                    storage_testing::errors::TestingError::ServiceStartupFailed {
                        reason: "one or more integration tests failed".to_string(),
                    },
                );
            }
            Ok(())
        }
        HarnessCommand::Cleanup { dry_run } => {
            require_root_if_needed(dry_run)?;
            let outcomes = lab::cleanup_all(ExecuteOptions { dry_run })?;
            for outcome in outcomes {
                if outcome.executed {
                    println!("{}", outcome.command);
                }
            }
            Ok(())
        }
    }
}
