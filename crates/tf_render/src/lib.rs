//! Crate: tf_render — Rendering engine and graphics abstraction.

pub mod camera;
pub mod debug_ui;
pub mod hud;
pub mod world;

use bevy::prelude::*;

pub use camera::IsometricCamera;
pub use world::{IsometricRoot, WorldRenderConfig, YSort};

/// Main render plugin for M1 world and HUD placeholders.
pub struct TideAndFortuneRenderPlugin;

impl Plugin for TideAndFortuneRenderPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			camera::CameraPlugin,
			world::WorldPlugin,
			debug_ui::DebugUiPlugin,
			hud::HudPlugin,
		));
	}
}