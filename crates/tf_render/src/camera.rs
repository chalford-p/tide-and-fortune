use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use glam::Vec2;
use tf_simulation::ship::{PlayerShip, ShipVelocity};

pub const WORLD_RENDER_LAYER: usize = 0;
pub const HUD_RENDER_LAYER: usize = 1;

/// Marker for the main isometric camera.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct IsometricCamera;

/// Marker for the HUD camera (not affected by isometric projection).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct HudCamera;

/// Camera tuning values for follow behavior and world clamping.
#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraFollowConfig {
    pub lookahead_seconds: f32,
    pub smoothing: f32,
    pub world_min: Vec2,
    pub world_max: Vec2,
    pub zoom_scale: f32,
}

impl Default for CameraFollowConfig {
    fn default() -> Self {
        Self {
            lookahead_seconds: 0.45,
            smoothing: 8.0,
            world_min: Vec2::ZERO,
            world_max: Vec2::splat(10_000.0),
            zoom_scale: 0.8,
        }
    }
}

/// Converts simulation world space into 2:1 isometric screen space.
pub fn world_to_isometric(world: Vec2) -> Vec2 {
    // Keep this projection in sync with `world::setup_world` root transform.
    Vec2::new(world.x - world.y * 0.5, world.x + world.y * 0.5)
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraFollowConfig>()
            .add_systems(Startup, (spawn_isometric_camera, spawn_hud_camera))
            .add_systems(Update, follow_player_ship);
    }
}

fn spawn_isometric_camera(mut commands: Commands, config: Res<CameraFollowConfig>) {
    let world_center = (config.world_min + config.world_max) * 0.5;
    let iso_center = world_to_isometric(world_center);

    let mut projection = OrthographicProjection::default_2d();
    projection.scale = config.zoom_scale;
    projection.near = -2_000.0;
    projection.far = 2_000.0;

    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        RenderLayers::layer(WORLD_RENDER_LAYER),
        projection,
        Transform::from_xyz(iso_center.x, iso_center.y, 999.0)
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_4))
            .with_scale(Vec3::new(std::f32::consts::SQRT_2 * 0.5, std::f32::consts::SQRT_2, 1.0)),
        IsometricCamera,
    ));
}

fn spawn_hud_camera(mut commands: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = 1.0;
    projection.near = -2_000.0;
    projection.far = 2_000.0;

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: bevy::render::camera::ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(HUD_RENDER_LAYER),
        projection,
        Transform::from_xyz(0.0, 0.0, 1000.0),
        HudCamera,
    ));
}

fn follow_player_ship(
    time: Res<Time>,
    config: Res<CameraFollowConfig>,
    player_q: Query<(&Transform, Option<&ShipVelocity>), With<PlayerShip>>,
    mut camera_q: Query<&mut Transform, (With<IsometricCamera>, Without<PlayerShip>)>,
) {
    let Ok((player_tf, player_velocity)) = player_q.get_single() else {
        return;
    };

    let Ok(mut camera_tf) = camera_q.get_single_mut() else {
        return;
    };

    let world_pos = Vec2::new(player_tf.translation.x, player_tf.translation.y);
    let lookahead = player_velocity
        .map(|velocity| velocity.linvel * config.lookahead_seconds)
        .unwrap_or(Vec2::ZERO);

    let world_target = clamp_world(world_pos + lookahead, config.world_min, config.world_max);
    let iso_target = world_to_isometric(world_target);

    let blend = 1.0 - (-config.smoothing * time.delta_secs()).exp();
    let blended_xy =
        Vec2::new(camera_tf.translation.x, camera_tf.translation.y).lerp(iso_target, blend);

    camera_tf.translation.x = blended_xy.x;
    camera_tf.translation.y = blended_xy.y;
}

fn clamp_world(position: Vec2, min: Vec2, max: Vec2) -> Vec2 {
    position.clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(left: Vec2, right: Vec2) {
        assert!((left.x - right.x).abs() < 1e-5);
        assert!((left.y - right.y).abs() < 1e-5);
    }

    #[test]
    fn converts_world_to_2_to_1_isometric() {
        let iso = world_to_isometric(Vec2::new(10.0, 4.0));
        approx_eq(iso, Vec2::new(8.0, 12.0));
    }

    #[test]
    fn clamps_world_targets_to_bounds() {
        let clamped = clamp_world(
            Vec2::new(12_000.0, -100.0),
            Vec2::ZERO,
            Vec2::splat(10_000.0),
        );
        approx_eq(clamped, Vec2::new(10_000.0, 0.0));
    }
}
