use gh_workflow::*;

use super::{runners, steps, steps::FluentBuilder, vars::WorkflowInput};

pub(crate) fn wezel() -> Workflow {
    let mut dispatch = WorkflowDispatch::default();
    for name in ["run_id", "experiment_name", "commit_sha", "project_dir"] {
        dispatch = dispatch.add_input(name, WorkflowInput::string(name, None).input());
    }
    steps::named::workflow()
        .permissions(Permissions::default().contents(Level::Read))
        .on(Event::default().workflow_dispatch(dispatch))
        .concurrency(
            Concurrency::default()
                .group("wezel-run-${{ inputs.run_id }}")
                .cancel_in_progress(false),
        )
        .add_job("macos", assigned_run("macos", runners::MAC_DEFAULT))
        .add_job("linux", assigned_run("linux", runners::LINUX_DEFAULT))
}

fn assigned_run(platform: &str, runner: runners::Runner) -> Job {
    let checkout = Step::<Use>::from(steps::checkout_repo().with_ref("${{ inputs.commit_sha }}"))
        .add_with(("persist-credentials", false));
    Job::default()
        .permissions(Permissions::default().contents(Level::Read))
        .runs_on(runner)
        .timeout_minutes(60u32)
        .cond(Expression::new(format!(
            "github.repository == 'zed-industries/zed' && inputs.project_dir == '.' && inputs.experiment_name == 'release-binary-size-{platform}'"
        )))
        .add_env(("ASSIGNED_COMMIT", "${{ inputs.commit_sha }}"))
        .add_env(("CARGO_TARGET_DIR", "target"))
        .add_step(
            Step::new("Validate assigned commit")
                .run(r#"[[ "$ASSIGNED_COMMIT" =~ ^[0-9a-f]{40}$ ]]"#),
        )
        .add_step(checkout)
        .add_step(Step::new("Verify assigned checkout").run(
            "test \"$(git rev-parse HEAD)\" = \"$ASSIGNED_COMMIT\"",
        ))
        .add_step(steps::cache_rust_dependencies_namespace())
        .when(platform == "linux", |job| {
            steps::install_linux_dependencies(steps::use_clang(job))
        })
        .when(platform == "macos", |job| job.add_step(steps::download_wasi_sdk()))
        .add_step(Step::new("Ensure zstd is available").run(match platform {
            "macos" => "command -v zstd || brew install --force-bottle zstd",
            _ => "command -v zstd || sudo apt-get install -y zstd",
        }))
        .add_step(
            Step::new("Run assigned Wezel experiment")
                .uses("wezel-build", "gh-action", "384b808f1de753a22abfadfe932452d0166c316f")
                .add_with(("wezel-version", "v0.1.5-pre-pre.20260916103109+5a021a6"))
                .add_with(("token", "${{ secrets.WEZEL_RUNNER_TOKEN }}"))
                .add_with(("run-id", "${{ inputs.run_id }}"))
                .add_with(("experiment-name", "${{ inputs.experiment_name }}"))
                .add_with(("project-dir", "${{ inputs.project_dir }}")),
        )
}
