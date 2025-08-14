use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct HclOverlayConfig {
    pub enabled: bool,
    pub max_vars: usize,
    pub max_recent: usize,
}

pub struct HclOverlayPlugin;

impl Plugin for HclOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HclOverlayConfig>();
        // No-UI overlay: logging is already handled in HclPlugin via runtime
    }
} 