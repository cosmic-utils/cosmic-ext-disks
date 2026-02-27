use storage_testing::harness;
use which::which;

#[test]
fn harness_plan_contains_privileged_run() {
    let runtime = if which("podman").is_ok() {
        "podman"
    } else if which("docker").is_ok() {
        "docker"
    } else {
        return;
    };

    let plan = harness::plan_run("harness_smoke", runtime, false, true).unwrap();
    assert!(plan.steps.iter().any(|step| step.contains("--privileged")));
}
