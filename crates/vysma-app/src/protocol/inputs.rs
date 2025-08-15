use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Reflect)]
pub struct Direction { pub(crate) up: bool, pub(crate) down: bool, pub(crate) left: bool, pub(crate) right: bool }
impl Direction { pub(crate) fn is_none(&self) -> bool { !self.up && !self.down && !self.left && !self.right } }

#[derive(Debug, PartialEq, Clone, Reflect)]
pub enum Inputs { Direction(Direction), Delete }
impl Default for Inputs { fn default() -> Self { Inputs::Direction(Direction { up: false, down: false, left: false, right: false }) } }
impl MapEntities for Inputs { fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {} } 