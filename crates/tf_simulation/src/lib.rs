//! Crate: tf_simulation — Game simulation logic and ECS systems.

pub mod ship;
pub mod systems;

use bevy_ecs::prelude::Resource;
use tf_core::sailing::wind::WindField;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, Default)]
pub enum GameMode {
	#[default]
	Harbor,
	Sailing,
}

#[derive(Debug, Clone, Resource)]
pub struct WindFieldResource {
	pub field: WindField,
}

impl WindFieldResource {
	pub fn new(field: WindField) -> Self {
		Self { field }
	}
}