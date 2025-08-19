use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context};

use crate::util::copy_dir_all;

pub fn run(name: &str) -> anyhow::Result<()> {
	let root = PathBuf::from(name);
	if root.exists() { bail!("directory '{}' already exists", name); }
	let tmpl = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates/basic");
	if !tmpl.exists() { bail!("template missing: {}", tmpl.display()); }
	copy_dir_all(&tmpl, &root).context("copy template")?;
	fs::write(root.join(".gitignore"), "target/\n.DS_Store\n")?;
	println!("Scaffolded '{}'\n- assets/\n- README.md\n- .gitignore", name);
	Ok(())
}


