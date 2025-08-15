use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut)]
pub struct PlayerPosition(pub Vec2);

impl Ease for PlayerPosition {
	fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self> {
		FunctionCurve::new(Interval::UNIT, move |t| { PlayerPosition(Vec2::lerp(start.0, end.0, t)) })
	}
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub(crate) Color);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut)]
pub struct CursorPosition(pub Vec2);

impl Ease for CursorPosition {
	fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self> {
		FunctionCurve::new(Interval::UNIT, move |t| { CursorPosition(Vec2::lerp(start.0, end.0, t)) })
	}
} 