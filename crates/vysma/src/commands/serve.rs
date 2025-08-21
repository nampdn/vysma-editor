use anyhow::Result;

pub fn run(_gui: bool) -> Result<()> {
	use std::time::Duration;
	use vysma_app::common::cli::{Cli as AppCli, Mode};
	use vysma_hcl::hcl::HclPlugin;
	let app_cli = AppCli { mode: Some(Mode::Server) };
	let mut app = app_cli.build_app(Duration::from_millis(50), false);
	app.add_plugins(HclPlugin);
	app.run();
	Ok(())
}


