use storage_testing::image_lab;

#[test]
fn lab_create_plan_contains_truncate_step() {
    let plan = image_lab::plan_create("2disk", true).unwrap();
    assert!(plan.steps.iter().any(|step| step.contains("truncate -s")));
}
