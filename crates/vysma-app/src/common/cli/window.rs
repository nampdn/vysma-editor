use bevy::window::{PresentMode, Window, WindowPlugin};
use bevy::prelude::default;

#[cfg(feature = "gui")]
pub fn window_plugin() -> WindowPlugin {
	WindowPlugin {
		primary_window: Some(Window {
			title: "Vysma Editor".to_string(),
			resolution: (1280.0, 720.0).into(),
			present_mode: PresentMode::AutoVsync,
			prevent_default_event_handling: true,
			..Default::default()
		}),
		..default()
	}
} 