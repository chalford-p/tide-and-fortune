//! Crate: tf_client — Game client binary entrypoint.

use bevy::prelude::*;
use glam::Vec2 as CoreVec2;
use tf_core::sailing::wind::{WindField, WindFieldConfig};
use tf_render::TideAndFortuneRenderPlugin;
use tf_simulation::ship::{PlayerShip, SailState, ShipBundle, ShipVelocity};
use tf_simulation::systems::player_input::player_input_system;
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
		.add_plugins(TideAndFortuneRenderPlugin)
		.add_systems(Startup, spawn_player_ship)
		.add_systems(
			Update,
			(
				update_wind_field_system,
				player_input_system,
				sailing_physics_system.after(player_input_system),
				log_player_state_system.after(sailing_physics_system),
			),
		)
		.run();
}

fn spawn_player_ship(mut commands: Commands) {
	let mut ship = ShipBundle::default();
	ship.transform.translation = Vec3::new(5_000.0, 5_000.0, 0.0);
	commands.spawn(ship);
}

fn update_wind_field_system(time: Res<Time>, mut wind: ResMut<WindFieldResource>) {
	wind.field.update(time.elapsed_secs());
}

fn log_player_state_system(
	time: Res<Time>,
	wind: Res<WindFieldResource>,
	player_q: Query<(&Transform, Option<&ShipVelocity>, Option<&SailState>), With<PlayerShip>>,
	mut log_accumulator: Local<f32>,
) {
	*log_accumulator += time.delta_secs();
	if *log_accumulator < 1.0 {
		return;
	}
	*log_accumulator -= 1.0;

	let Ok((transform, velocity, sail_state)) = player_q.get_single() else {
		return;
	};

	let position = transform.translation;
	let linvel = velocity.map(|v| v.linvel).unwrap_or(CoreVec2::ZERO);
	let sample_pos = CoreVec2::new(position.x, position.y);
	let wind_vec = wind.field.at(sample_pos);

	if let Some(sail_state) = sail_state {
		info!(
			"player vel=({:.2}, {:.2}) wind=({:.2}, {:.2}) sail tier={:?} active={:?} trim={:?}",

			linvel.x,
			linvel.y,
			wind_vec.x,
			wind_vec.y,
			sail_state.tier,
			sail_state.active_sails,
			sail_state.trim_per_sail,
		);
	} else {
		info!(
			"player vel=({:.2}, {:.2}) wind=({:.2}, {:.2}) sail=unavailable",
			linvel.x,
			linvel.y,
			wind_vec.x,
			wind_vec.y,
		);
	}
}