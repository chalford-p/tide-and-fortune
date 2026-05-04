//! Crate: tf_simulation — Game simulation logic and ECS systems.

pub mod ship;
pub mod systems;

use bevy_ecs::prelude::Resource;
use std::collections::VecDeque;
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

#[derive(Debug, Clone, Resource)]
pub struct DebugControlsState {
	pub enabled: bool,
	pub zero_wind_enabled: bool,
	console_lines: VecDeque<String>,
	max_console_lines: usize,
}

impl Default for DebugControlsState {
	fn default() -> Self {
		let mut state = Self {
			enabled: false,
			zero_wind_enabled: false,
			console_lines: VecDeque::new(),
			max_console_lines: 64,
		};
		state.push_console_line("Debug controls ready. Press Shift+C to open.");
		state
	}
}

impl DebugControlsState {
	pub fn push_console_line<S: Into<String>>(&mut self, message: S) {
		self.console_lines.push_back(message.into());
		while self.console_lines.len() > self.max_console_lines {
			self.console_lines.pop_front();
		}
	}

	pub fn console_lines(&self) -> impl Iterator<Item = &str> {
		self.console_lines.iter().map(String::as_str)
	}
}