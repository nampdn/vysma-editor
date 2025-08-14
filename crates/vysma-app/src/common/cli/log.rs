use bevy::log::{Level, LogPlugin};
use bevy::prelude::default;

pub fn log_plugin() -> LogPlugin {
	LogPlugin {
		level: Level::INFO,
		filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn".to_string(),
		..default()
	}
} 