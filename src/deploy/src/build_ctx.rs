use crate::{cfg::BuildProfile, Params};
use std::process::Command;
use util::cmd::Runner;

pub struct BuildCtx<'bctx> {
    params: &'bctx Params,
    runner: &'bctx util::cmd::Runner,
}

impl<'bctx> BuildCtx<'bctx> {
    pub(crate) fn new(params: &'bctx Params, runner: &'bctx Runner) -> Self {
        Self { params, runner }
    }

    pub(crate) fn runner(&self) -> &'bctx Runner {
        self.runner
    }

    pub(crate) fn cargo(&self) -> Command {
        let mut cmd = Command::new("cargo");
        cmd.env("CARGO_PROFILE_RELEASE_INCREMENTAL", "false");
        cmd.current_dir(&self.params.src);
        cmd
    }

    pub(crate) fn cargo_build(&self) -> Command {
        let mut cmd = self.cargo();
        cmd.arg("build");
        if let Some(target) = &self.params.cfg.build.target {
            cmd.args(&["--target", target]);
        }
        let profile = self.params.cfg.build.profile;
        if let BuildProfile::Release | BuildProfile::RelWithDebInfo = profile {
            cmd.arg("--release");
        }
        if let BuildProfile::RelWithDebInfo = profile {
            cmd.env("CARGO_PROFILE_RELEASE_DEBUG", "true");
        }
        cmd.arg("-Zunstable-options");
        cmd.arg("--out-dir").arg(self.params.build.join("jjs-out"));
        cmd.arg("--locked");
        cmd
    }
}
