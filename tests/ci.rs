use gh_workflow::*;
use serde_json::json;

#[test]
fn main() {
    let runs_on_major_os_matrix = |job: Job| {
        job.runs_on("${{ matrix.os }}").strategy(
            Strategy::default()
                .matrix(json!({ "os": ["windows-latest", "macos-latest", "ubuntu-latest"] })),
        )
    };
    let install_native_dependencies = |job: Job| {
        job.add_step(Step::new("Install native dependencies")
        .run("sudo apt-get update; sudo apt-get install --no-install-recommends libasound2-dev libudev-dev libfontconfig1-dev")
        .if_condition(Expression::new("runner.os == 'Linux'")))
    };
    let install_tool = |job: Job, (name, tool): (&str, &str)| {
        job.add_step(
            Step::new(name)
                .uses("taiki-e", "install-action", "v2")
                .with(("tool", tool)),
        )
    };
    let install_tools = |job: Job, lines: Vec<(&str, &str)>| {
        lines
            .iter()
            .fold(job, |job, tool| job.apply_with(install_tool, *tool))
    };

    let pg_env = |step: Step<Run>| {
        step.env(("PGSERVICE", "${{ steps.postgres.outputs.service-name }}"))
            .env((
                "DATABASE_URL",
                "${{ steps.postgres.outputs.connection-uri }}",
            ))
    };
    let setup_database = |job: Job| {
        job.add_step(
            Step::new("Setup PostgreSQL")
                .uses("ikalnytskyi", "action-setup-postgres", "v8")
                .with(("username", "postgres"))
                .add_with(("password", "postgres"))
                .add_with(("database", "kreqo"))
                .add_with(("port", 8080))
                .add_with(("postgres-version", 18))
                .id("postgres"),
        )
        .apply_with(install_tool, ("Install sqlx-cli", "sqlx-cli"))
        .add_step(
            Step::new("Run sqlx migrate run")
                .run("sqlx migrate run --source server/migrations")
                .apply(pg_env),
        )
    };

    let workflow = Workflow::default()
        .name("CI")
        .on(Event::default()
            .push(Push::default().add_branch("main"))
            .pull_request(PullRequest::default().add_branch("main")))
        .add_job(
            "fmt",
            Job::new("formatting")
                .add_step(Step::checkout())
                .add_step(Step::toolchain().add_nightly().add_fmt().cache(true))
                .add_step(Step::new("Run cargo-machete").uses("bnjbvr", "cargo-machete", "main"))
                .apply_with(
                    install_tools,
                    vec![
                        ("Install cargo-sort", "cargo-sort"),
                        ("Install taplo", "taplo"),
                    ],
                )
                .add_step(
                    Step::new("Run cargo-sort")
                        .run("cargo +nightly sort --workspace --grouped --check --check-format"),
                )
                .add_step(Step::new("Run taplo fmt").run("taplo fmt --check --diff"))
                .add_step(Step::new("Run cargo fmt").run("cargo +nightly fmt --all -- --check")),
        )
        .add_job(
            "sqlx_offline",
            Job::new("sqlx offline")
                .add_step(Step::checkout())
                .add_step(Step::toolchain().add_nightly().cache(true))
                .apply(setup_database)
                .apply(install_native_dependencies)
                .add_step(
                    Step::new("Run cargo sqlx prepare")
                        .run("cargo +nightly sqlx prepare --check --workspace")
                        .apply(pg_env),
                ),
        )
        .add_job(
            "lint",
            Job::new("linting")
                .runs_on("${{ matrix.os }}")
                .apply(runs_on_major_os_matrix)
                .add_step(Step::checkout())
                .add_step(Step::toolchain().add_nightly().add_clippy().cache(true))
                .apply(install_native_dependencies)
                .add_step(
                    Step::new("Run cargo clippy").run("cargo +nightly clippy -- -D warnings"),
                ),
        )
        .add_job(
            "test",
            Job::new("testing")
                .runs_on("${{ matrix.os }}")
                .apply(runs_on_major_os_matrix)
                .add_step(Step::checkout())
                .add_step(Step::toolchain().cache(true))
                .apply(install_native_dependencies)
                .add_step(
                    Step::new("Run cargo test")
                        .run("cargo +nightly test")
                        .env(("RUST_BACKTRACE", "full")),
                ),
        );

    workflow.generate().expect("workflow should generate");
}

trait ExternMethod
where
    Self: Sized,
{
    fn apply<F>(self, method: F) -> Self
    where
        F: Fn(Self) -> Self,
    {
        method(self)
    }
    fn apply_with<F, O>(self, method: F, options: O) -> Self
    where
        F: Fn(Self, O) -> Self,
    {
        method(self, options)
    }
}

impl ExternMethod for Job {}
impl<A> ExternMethod for Step<A> {}
