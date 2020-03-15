use crate::ci::{detect_build_type, DeployKind};
use std::process::Command;
use structopt::StructOpt;
use util::cmd::{CommandExt, Runner};

#[derive(StructOpt)]
pub(crate) struct RawBuildOpts {
    /// enable things that are not required for running tests
    #[structopt(long)]
    full: bool,
    /// Enable docker
    #[structopt(long)]
    docker: bool,
    /// Setup (useful for development)
    #[structopt(long)]
    setup: bool,
    /// Debian packages
    #[structopt(long)]
    deb: bool,
    /// Additional options to pass to configure
    #[structopt(long = "configure-opt")]
    configure: Vec<String>,
    /// Build all docs
    #[structopt(long)]
    docs: bool,
}

struct BuildOpts(RawBuildOpts);

impl BuildOpts {
    fn full(&self) -> bool {
        let deploy_wants_full = detect_build_type().is_deploy()
            && (detect_build_type().deploy_info() != Some(DeployKind::Man));
        deploy_wants_full || self.0.full
    }

    fn should_build_deb(&self) -> bool {
        detect_build_type().is_pr_e2e()
            || detect_build_type()
                .deploy_info()
                .contains(&DeployKind::Docker)
            || self.0.deb
    }

    fn should_build_man(&self) -> bool {
        let bt = detect_build_type();
        bt.deploy_info().contains(&DeployKind::Man) || self.0.full || self.0.docs
    }

    fn should_build_docker(&self) -> bool {
        self.0.docker
            || detect_build_type()
                .deploy_info()
                .contains(&DeployKind::Docker)
    }

    fn raw(&self) -> &RawBuildOpts {
        &self.0
    }
}

pub(crate) fn task_build(opts: RawBuildOpts, runner: &Runner) -> anyhow::Result<()> {
    let opts = BuildOpts(opts);
    std::fs::File::create("./target/.jjsbuild").unwrap();
    let mut cmd = Command::new("../configure");
    cmd.current_dir("target");
    cmd.arg("--out=/opt/jjs");
    cmd.arg("--enable-json-schema");

    if opts.full() || opts.should_build_deb() {
        cmd.arg("--enable-deb");
        let bt = crate::ci::detect_build_type();
        if bt.is_pr_e2e() {
            cmd.arg("--with-deb-opt=--uncompressed");
        }
    }
    if detect_build_type().is_deploy() {
        cmd.arg("--optimize");
    }
    // useful for easily starting up & shutting down
    // required for docker compose
    if opts.should_build_docker() {
        cmd.arg("--enable-docker");
    }
    if opts.full() {
        cmd.arg("--enable-archive");
        cmd.arg("--enable-extras");
    }
    if opts.should_build_man() {
        cmd.arg("--disable-core");
        cmd.arg("--disable-tools");
        cmd.arg("--enable-api-doc");
    } else {
        cmd.arg("--disable-man");
    }

    for opt in &opts.raw().configure {
        cmd.arg(opt);
    }
    cmd.try_exec()?;

    Command::new("make").current_dir("target").try_exec()?;

    runner.exit_if_errors();

    if opts.raw().setup {
        println!("running setup");
        Command::new("/opt/jjs/bin/jjs-setup")
            .arg("--data-dir=/tmp/jjs")
            .arg("--install-dir=/opt/jjs")
            .arg("--db-url=postgres://jjs:internal@localhost:5432/jjs")
            .arg("--drop-db")
            .arg("--force")
            .arg("--sample-contest")
            .arg("--symlink-config")
            .arg("--setup-config")
            .arg("--toolchains")
            .run_on(runner);
    }
    Ok(())
}
