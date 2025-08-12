use bevy::prelude::*;

use super::runtime::HclRuntime;

#[derive(Resource, Default)]
pub struct HclOverlayConfig {
    pub enabled: bool,
    pub max_vars: usize,
    pub max_recent: usize,
}

#[derive(Component)]
struct HclOverlayText;

#[derive(Component)]
struct HclOverlayCamera;

pub struct HclOverlayPlugin;

impl Plugin for HclOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HclOverlayConfig>();
        app.add_systems(Startup, setup_overlay);
        app.add_systems(Update, (update_overlay_text, position_overlay_text));
    }
}

fn setup_overlay(mut commands: Commands, mut cfg: ResMut<HclOverlayConfig>) {
    if !cfg.enabled {
        // default on for first run
        cfg.enabled = true;
        cfg.max_vars = 8;
        cfg.max_recent = 5;
    }
    // 2D camera for overlay
    commands.spawn((Camera2dBundle::default(), HclOverlayCamera));
    // Text2d near top-left; exact position will be adjusted each frame
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "HCL",
                TextStyle { font_size: 14.0, color: Color::WHITE, ..Default::default() },
            ),
            transform: Transform::from_translation(Vec3::new(10.0, 10.0, 0.0)),
            ..Default::default()
        },
        HclOverlayText,
    ));
}

fn update_overlay_text(
    cfg: Option<Res<HclOverlayConfig>>,
    runtime: Option<Res<HclRuntime>>,
    mut q_text: Query<&mut Text, With<HclOverlayText>>,
) {
    let Some(cfg) = cfg else { return; };
    if !cfg.enabled { return; }
    let Ok(mut text) = q_text.get_single_mut() else { return; };
    if let Some(rt) = runtime {
        let line = rt.overlay_line(cfg.max_vars, cfg.max_recent);
        *text = Text::from_section(
            line,
            TextStyle { font_size: 14.0, color: Color::WHITE, ..Default::default() },
        );
    }
}

fn position_overlay_text(
    cfg: Option<Res<HclOverlayConfig>>,
    windows: Query<&Window>,
    mut q_tf: Query<&mut Transform, With<HclOverlayText>>,
) {
    let Some(cfg) = cfg else { return; };
    if !cfg.enabled { return; }
    let Ok(window) = windows.get_single() else { return; };
    let Ok(mut tf) = q_tf.get_single_mut() else { return; };
    // Place near top-left in 2D coordinates (window center at (0,0))
    let x = -window.width() * 0.5 + 10.0;
    let y = window.height() * 0.5 - 20.0;
    tf.translation = Vec3::new(x, y, 0.0);
} 