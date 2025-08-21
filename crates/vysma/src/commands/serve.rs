use anyhow::Result;

pub fn run(_gui: bool) -> Result<()> {
	use std::time::Duration;
	use vysma_app::common::cli::{Cli as AppCli, Mode};
	use vysma_hcl::hcl::{HclPlugin, auto_discover_hcl_scenes};
    use vysma_app::prelude as prelude;
	let app_cli = AppCli { mode: Some(Mode::Server) };
	let mut app = app_cli.build_app(Duration::from_millis(50), false);
	app.add_plugins(HclPlugin);
    // Use auto-discovery instead of hardcoded paths
    app.add_systems(prelude::Startup, auto_discover_hcl_scenes());
	app.run();
	Ok(())
}


