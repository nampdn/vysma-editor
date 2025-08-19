use anyhow::{bail, Context};

use crate::util::workspace_root;

pub fn run(client_id: Option<u64>, _connect: &Option<String>, gui: bool) -> anyhow::Result<()> {
	let mut cmd = std::process::Command::new("cargo");
	cmd.arg("run").arg("-p").arg("bevy-in-app").arg("--");
	cmd.arg("client");
	if let Some(id) = client_id { cmd.arg("-c").arg(id.to_string()); }
	if gui { cmd.arg("--features").arg("gui"); }
	cmd.env("RUST_LOG", "info");
	cmd.current_dir(workspace_root());
	let status = cmd.status().context("run client")?;
	if !status.success() { bail!("client exited with status {:?}", status.code()); }
	Ok(())
}


