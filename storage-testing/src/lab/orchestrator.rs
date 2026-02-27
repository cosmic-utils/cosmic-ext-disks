use crate::cmd::CommandOutcome;
use crate::errors::Result;
use crate::lab::image;
use crate::spec;

#[derive(Debug, Clone, Copy, Default)]
pub struct ExecuteOptions {
    pub dry_run: bool,
}

fn run_and_collect(plan: image::Plan) -> Result<Vec<CommandOutcome>> {
    image::execute_plan(&plan)
}

pub fn create(spec_name: &str, opts: ExecuteOptions) -> Result<Vec<CommandOutcome>> {
    run_and_collect(image::plan_create(spec_name, opts.dry_run)?)
}

pub fn prepare(spec_name: &str, opts: ExecuteOptions) -> Result<Vec<CommandOutcome>> {
    run_and_collect(image::plan_prepare(spec_name, opts.dry_run)?)
}

pub fn attach(spec_name: &str, opts: ExecuteOptions) -> Result<Vec<CommandOutcome>> {
    let outcomes = run_and_collect(image::plan_attach(spec_name, opts.dry_run)?)?;
    if !opts.dry_run {
        let loops: Vec<String> = outcomes
            .iter()
            .filter(|result| result.executed)
            .map(|result| result.stdout.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect();
        image::record_attach_state(spec_name, &loops)?;
    }
    Ok(outcomes)
}

pub fn mount(spec_name: &str, opts: ExecuteOptions) -> Result<Vec<CommandOutcome>> {
    let outcomes = run_and_collect(image::plan_mount(spec_name, opts.dry_run)?)?;
    if !opts.dry_run {
        let loaded_spec = spec::load_by_name(spec_name)?;
        let mount_points = loaded_spec
            .mounts
            .iter()
            .map(|mount| mount.mount_point.clone())
            .collect::<Vec<_>>();
        image::record_mount_state(spec_name, &mount_points)?;
    }
    Ok(outcomes)
}

pub fn unmount(spec_name: &str, opts: ExecuteOptions) -> Result<Vec<CommandOutcome>> {
    run_and_collect(image::plan_unmount(spec_name, opts.dry_run)?)
}

pub fn detach(spec_name: &str, opts: ExecuteOptions) -> Result<Vec<CommandOutcome>> {
    run_and_collect(image::plan_detach(spec_name, opts.dry_run)?)
}

pub fn destroy(spec_name: &str, opts: ExecuteOptions) -> Result<Vec<CommandOutcome>> {
    run_and_collect(image::plan_destroy(spec_name, opts.dry_run)?)
}

pub fn cleanup(spec_name: &str, opts: ExecuteOptions) -> Result<Vec<CommandOutcome>> {
    run_and_collect(image::plan_cleanup(spec_name, opts.dry_run)?)
}

pub fn cleanup_all(opts: ExecuteOptions) -> Result<Vec<CommandOutcome>> {
    run_and_collect(image::plan_cleanup_all(opts.dry_run)?)
}

pub fn setup_group(spec_name: &str, opts: ExecuteOptions) -> Result<()> {
    create(spec_name, opts)?;
    prepare(spec_name, opts)?;
    attach(spec_name, opts)?;
    mount(spec_name, opts)?;
    Ok(())
}

pub fn teardown_group(spec_name: &str, opts: ExecuteOptions) -> Result<()> {
    let _ = unmount(spec_name, opts);
    let _ = detach(spec_name, opts);
    let _ = destroy(spec_name, opts);
    Ok(())
}
