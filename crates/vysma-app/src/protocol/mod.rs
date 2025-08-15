pub mod components;
pub mod inputs;

use bevy::prelude::*;
use lightyear::prelude::*;
use vysma_hcl::hcl::net::HclSceneBlob;

pub use components::{PlayerId, PlayerPosition, PlayerColor, CursorPosition};
pub use inputs::{Direction, Inputs};

pub(crate) struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<Inputs>();
		app.register_component::<PlayerId>().add_prediction(PredictionMode::Once).add_interpolation(InterpolationMode::Once);
		app.register_component::<Name>().add_prediction(PredictionMode::Once).add_interpolation(InterpolationMode::Once);
		app.register_component::<PlayerPosition>().add_prediction(PredictionMode::Full).add_interpolation(InterpolationMode::Full).add_linear_interpolation_fn();
		app.register_component::<PlayerColor>().add_prediction(PredictionMode::Once).add_interpolation(InterpolationMode::Once);
		app.register_component::<CursorPosition>().add_prediction(PredictionMode::Full).add_interpolation(InterpolationMode::Full).add_linear_interpolation_fn();
		app.register_component::<HclSceneBlob>().add_prediction(PredictionMode::Once).add_interpolation(InterpolationMode::Once);
	}
} 