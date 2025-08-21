use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context};

use crate::util::copy_dir_all;

pub fn run(name: &str, template: &str, overwrite: bool) -> anyhow::Result<()> {
	let root = PathBuf::from(name);
	if root.exists() {
		if !overwrite { bail!("directory '{}' already exists", name); }
		std::fs::remove_dir_all(&root).with_context(|| format!("remove existing dir: {}", root.display()))?;
	}
	let tmpl = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates").join(template);
	if !tmpl.exists() { bail!("template missing: {}", tmpl.display()); }
	copy_dir_all(&tmpl, &root).context("copy template")?;
	fs::write(root.join(".gitignore"), "target/\n.DS_Store\n")?;
	println!("Scaffolded '{}' from '{}'\n- assets/\n- README.md\n- .gitignore", name, template);
	Ok(())
}


