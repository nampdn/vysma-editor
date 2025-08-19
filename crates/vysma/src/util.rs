use std::fs;
use std::path::{Path, PathBuf};

pub fn workspace_root() -> PathBuf {
	let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	// crates/vysma → workspace root
	here.join("../..").canonicalize().unwrap_or(here)
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
	fs::create_dir_all(dst)?;
	for entry in fs::read_dir(src)? {
		let entry = entry?;
		let meta = entry.file_type()?;
		let to = dst.join(entry.file_name());
		if meta.is_dir() {
			copy_dir_all(&entry.path(), &to)?;
		} else if meta.is_file() {
			if let Some(parent) = to.parent() { fs::create_dir_all(parent)?; }
			fs::copy(entry.path(), &to)?;
		}
	}
	Ok(())
}


