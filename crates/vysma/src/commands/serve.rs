use anyhow::{bail, Context};

use crate::util::workspace_root;

pub fn run(gui: bool) -> anyhow::Result<()> {
	let mut cmd = std::process::Command::new("cargo");
	cmd.arg("run").arg("-p").arg("bevy-in-app").arg("--bin").arg("bevy-in-app");
	if gui { cmd.arg("--features").arg("gui"); }
	cmd.arg("--");
	cmd.arg("server");
	cmd.env("RUST_LOG", "info");
	cmd.current_dir(workspace_root());
	let status = cmd.status().context("run server")?;
	if !status.success() { bail!("server exited with status {:?}", status.code()); }
	Ok(())
}


