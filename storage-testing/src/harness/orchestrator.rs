use std::fs;
use std::sync::Arc;

use crate::artifacts;
use crate::errors::{Result, TestingError};
use crate::ledger;
use crate::lab::orchestrator as lab;
use crate::lab::orchestrator::ExecuteOptions;
use crate::spec;
use crate::tests::{self, HarnessContext, TestRef};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::{Duration, Instant, timeout};

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub suite: Option<String>,
    pub test_id: Option<String>,
    pub max_parallel_groups: usize,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaseStatus {
    Passed,
    Failed(String),
    Skipped(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResultRecord {
    pub id: String,
    pub spec: String,
    pub status: CaseStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResultRecord {
    pub spec: String,
    pub tests: Vec<TestResultRecord>,
    pub setup_error: Option<String>,
    pub teardown_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub groups: Vec<GroupResultRecord>,
}

impl RunSummary {
    pub fn has_failures(&self) -> bool {
        self.groups.iter().any(|group| {
            group.setup_error.is_some()
                || group.teardown_error.is_some()
                || group
                    .tests
                    .iter()
                    .any(|test| matches!(test.status, CaseStatus::Failed(_)))
        })
    }
}

async fn execute_test_case(spec: &str, test: TestRef, ctx: HarnessContext) -> TestResultRecord {
    println!("[{}] START {}", spec, test.id());
    let started = Instant::now();
    let timeout_secs = std::env::var("STORAGE_TESTING_TEST_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(10);

    let status = match timeout(Duration::from_secs(timeout_secs), test.execute(&ctx)).await {
        Ok(result) => match result {
            Ok(_) => CaseStatus::Passed,
            Err(TestingError::TestSkipped { reason }) => CaseStatus::Skipped(reason),
            Err(error) => CaseStatus::Failed(error.to_string()),
        },
        Err(_) => CaseStatus::Skipped(format!(
            "test timed out after {}s",
            timeout_secs
        )),
    };

    let elapsed = started.elapsed().as_millis();
    match &status {
        CaseStatus::Passed => println!("[{}] PASS {} ({} ms)", spec, test.id(), elapsed),
        CaseStatus::Skipped(reason) => {
            println!("[{}] SKIP {} ({} ms): {}", spec, test.id(), elapsed, reason)
        }
        CaseStatus::Failed(reason) => {
            println!("[{}] FAIL {} ({} ms): {}", spec, test.id(), elapsed, reason)
        }
    }

    TestResultRecord {
        id: test.id().to_string(),
        spec: spec.to_string(),
        status,
    }
}

async fn run_group(spec: String, tests: Vec<TestRef>, dry_run: bool) -> GroupResultRecord {
    let mut group = GroupResultRecord {
        spec: spec.clone(),
        tests: Vec::new(),
        setup_error: None,
        teardown_error: None,
    };

    let opts = ExecuteOptions { dry_run };
    let ctx = HarnessContext { dry_run };

    if let Err(error) = lab::setup_group(&spec, opts) {
        group.setup_error = Some(error.to_string());
        return group;
    }

    if let Err(error) = configure_group_environment(&spec) {
        group.setup_error = Some(error.to_string());
        let _ = lab::teardown_group(&spec, opts);
        return group;
    }

    for test in tests {
        group
            .tests
            .push(execute_test_case(&spec, Arc::clone(&test), ctx.clone()).await);
    }

    group.tests.sort_by(|left, right| left.id.cmp(&right.id));

    if let Err(error) = lab::teardown_group(&spec, opts) {
        group.teardown_error = Some(error.to_string());
    }

    group
}

fn set_env_if_some(name: &str, value: Option<String>) {
    if let Some(value) = value
        && !value.trim().is_empty()
    {
        unsafe {
            std::env::set_var(name, value);
        }
    }
}

fn configure_group_environment(spec_name: &str) -> Result<()> {
    let loaded_spec = spec::load_by_name(spec_name)?;
    let state = ledger::load(spec_name)?;

    let mounted_partitions: Vec<String> = loaded_spec
        .mounts
        .iter()
        .enumerate()
        .filter_map(|(index, mount)| {
            state
                .mapped_partitions
                .iter()
                .filter(|value| value.ends_with(&mount.partition_ref))
                .nth(index)
                .cloned()
        })
        .collect();

    let mounted_partition = mounted_partitions.first().cloned();
    let free_partitions: Vec<String> = state
        .mapped_partitions
        .iter()
        .filter(|value| !mounted_partitions.iter().any(|mounted| mounted == *value))
        .cloned()
        .collect();

    let first_loop = state.loop_devices.first().cloned();
    let first_loop_known = first_loop.clone();
    let second_loop = state.loop_devices.get(1).cloned();
    let first_partition = mounted_partition
        .clone()
        .or_else(|| state.mapped_partitions.first().cloned());
    let second_partition = free_partitions.first().cloned();
    let third_partition = free_partitions.get(1).cloned();
    let btrfs_source_partition = first_partition.clone().or(second_partition.clone());
    let backup_image_path = spec::artifacts_root(&loaded_spec)
        .join(format!("{}-backup.img", loaded_spec.name))
        .display()
        .to_string();
    let source_image_path = state.image_paths.first().cloned();
    let first_mount = loaded_spec.mounts.first().map(|mount| mount.mount_point.clone());

    set_env_if_some("STORAGE_TESTING_PARTITION_DISK", second_loop.clone().or(first_loop_known.clone()));
    set_env_if_some("STORAGE_TESTING_DISK_DEVICE", first_loop_known.clone());
    set_env_if_some("STORAGE_TESTING_MOUNT_DEVICE", first_partition.clone());
    set_env_if_some("STORAGE_TESTING_MOUNT_OPTIONS_DEVICE", first_partition.clone());
    set_env_if_some("STORAGE_TESTING_CHECK_DEVICE", first_partition.clone());
    set_env_if_some("STORAGE_TESTING_LUKS_DEVICE", second_partition.clone().or(first_partition.clone()));
    set_env_if_some("STORAGE_TESTING_PARTITION_DEVICE", second_partition.clone().or(first_partition.clone()));
    set_env_if_some("STORAGE_TESTING_LUKS_PASSPHRASE", Some("storage-test-passphrase".to_string()));
    set_env_if_some("STORAGE_TESTING_IMAGE_DEVICE", first_loop_known.clone());
    set_env_if_some("STORAGE_TESTING_IMAGE_PATH", Some(backup_image_path));
    set_env_if_some("STORAGE_TESTING_IMAGE_SOURCE_PATH", source_image_path);
    set_env_if_some("STORAGE_TESTING_MOUNT_POINT", first_mount.clone());
    set_env_if_some("STORAGE_TESTING_BTRFS_MOUNT", first_mount);
    set_env_if_some("STORAGE_TESTING_BTRFS_MEMBER_DEVICE", third_partition.clone());
    set_env_if_some("STORAGE_TESTING_BTRFS_SOURCE_DEVICE", btrfs_source_partition);
    set_env_if_some("STORAGE_TESTING_LVM_VG", Some("storage_test_vg".to_string()));

    if let (Some(part_a), Some(part_b)) = (second_partition.clone(), third_partition.clone()) {
        let md_array_name = format!("/dev/md/storage-testing-{}", std::process::id());
        let md_devices_json = format!("[\"{}\",\"{}\"]", part_a, part_b);
        set_env_if_some("STORAGE_TESTING_MD_ARRAY", Some(md_array_name));
        set_env_if_some("STORAGE_TESTING_MD_DEVICES_JSON", Some(md_devices_json));

        let lvm_pvs_json = format!("[\"{}\",\"{}\"]", part_a, part_b);
        set_env_if_some("STORAGE_TESTING_LVM_PVS_JSON", Some(lvm_pvs_json));
    }

    Ok(())
}

pub async fn run_all(config: RunConfig) -> Result<RunSummary> {
    let selected = filter_tests(
        tests::instantiate_tests(),
        config.suite.as_deref(),
        config.test_id.as_deref(),
    );
    if selected.is_empty() {
        return Err(TestingError::ServiceStartupFailed {
            reason: "no tests selected for current filters".to_string(),
        });
    }

    let artifact_dir = artifacts::run_dir("harness")?;

    let groups = group_by_spec(selected);
    let max_parallel = config.max_parallel_groups.max(1);
    let semaphore = Arc::new(Semaphore::new(max_parallel));
    let mut set = JoinSet::new();

    for (spec, tests) in groups {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|error| TestingError::ServiceStartupFailed {
                reason: format!("harness semaphore closed: {error}"),
            })?;
        let dry_run = config.dry_run;
        set.spawn(async move {
            let _permit = permit;
            run_group(spec, tests, dry_run).await
        });
    }

    let mut all_groups = Vec::new();
    while let Some(joined) = set.join_next().await {
        let group = joined.map_err(|error| TestingError::ServiceStartupFailed {
            reason: format!("harness worker task failed: {error}"),
        })?;
        all_groups.push(group);
    }

    all_groups.sort_by(|left, right| left.spec.cmp(&right.spec));

    let summary = RunSummary { groups: all_groups };

    for group in &summary.groups {
        let group_path = artifact_dir.join(format!("group-{}.log", sanitize_name(&group.spec)));
        let group_content =
            serde_json::to_string_pretty(group).map_err(|error| TestingError::LedgerIo {
                path: group_path.clone(),
                reason: error.to_string(),
            })?;
        fs::write(&group_path, group_content).map_err(|error| TestingError::LedgerIo {
            path: group_path,
            reason: error.to_string(),
        })?;

        for test in &group.tests {
            let test_path = artifact_dir.join(format!("test-{}.log", sanitize_name(&test.id)));
            let test_content =
                serde_json::to_string_pretty(test).map_err(|error| TestingError::LedgerIo {
                    path: test_path.clone(),
                    reason: error.to_string(),
                })?;
            fs::write(&test_path, test_content).map_err(|error| TestingError::LedgerIo {
                path: test_path,
                reason: error.to_string(),
            })?;
        }
    }

    let summary_path = artifact_dir.join("run-summary.json");
    let summary_content =
        serde_json::to_string_pretty(&summary).map_err(|error| TestingError::LedgerIo {
            path: summary_path.clone(),
            reason: error.to_string(),
        })?;
    fs::write(&summary_path, summary_content).map_err(|error| TestingError::LedgerIo {
        path: summary_path,
        reason: error.to_string(),
    })?;

    Ok(summary)
}

fn filter_tests(tests: Vec<TestRef>, suite: Option<&str>, test_id: Option<&str>) -> Vec<TestRef> {
    tests
        .into_iter()
        .filter(|test| suite.is_none_or(|value| test.suite() == value))
        .filter(|test| test_id.is_none_or(|value| test.id() == value))
        .collect()
}

fn group_by_spec(mut tests: Vec<TestRef>) -> Vec<(String, Vec<TestRef>)> {
    tests.sort_by(|left, right| {
        left.required_spec()
            .cmp(right.required_spec())
            .then(left.id().cmp(right.id()))
    });

    let mut groups: Vec<(String, Vec<TestRef>)> = Vec::new();
    for test in tests {
        let spec = test.required_spec().to_string();
        if let Some((_, group_tests)) = groups.iter_mut().find(|(value, _)| *value == spec) {
            group_tests.push(test);
        } else {
            groups.push((spec, vec![test]));
        }
    }
    groups
}

fn sanitize_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}
