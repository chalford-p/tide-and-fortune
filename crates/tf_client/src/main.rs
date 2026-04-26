//! Crate: tf_client — Game client binary entrypoint.

use bevy::prelude::*;
use tf_core::sailing::wind::{WindField, WindFieldConfig};
use tf_render::TideAndFortuneRenderPlugin;
use tf_simulation::ship::{Helm, ShipBundle};
use tf_simulation::systems::player_input::{player_input_system, HeadingChanged};
use tf_simulation::systems::sailing_physics::sailing_physics_system;
use tf_simulation::{GameMode, WindFieldResource};

fn main() {
	App::new()
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			primary_window: Some(Window {
				title: "Tide and Fortune".to_string(),
				resolution: (1280.0, 720.0).into(),
				..default()
			}),
			..default()
		}))
		.insert_resource(GameMode::Sailing)
		.insert_resource(WindFieldResource::new(WindField::new(WindFieldConfig {
			world_min: glam::Vec2::ZERO,
			world_max: glam::Vec2::splat(10_000.0),
			cell_size: 250.0,
			min_speed: 4.0,
			max_speed: 14.0,
			gust_strength: 0.25,
		})))
		.add_event::<HeadingChanged>()
		.add_plugins(TideAndFortuneRenderPlugin)
		.add_systems(Startup, spawn_player_ship)
		.add_systems(
			Update,
			(
				update_wind_field_system,
				player_input_system,
				sailing_physics_system.after(player_input_system),
			),
		)
		.run();
}

fn spawn_player_ship(mut commands: Commands) {
	let mut ship = ShipBundle::default();
	ship.transform.translation = Vec3::new(5_000.0, 5_000.0, 0.0);
	ship.helm = Helm {
		target_heading: 0.0,
		rudder_angle: 0.0,
	};

	commands.spawn(ship);
}

fn update_wind_field_system(time: Res<Time>, mut wind: ResMut<WindFieldResource>) {
	wind.field.update(time.elapsed_secs());
}