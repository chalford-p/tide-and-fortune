use bevy::prelude::*;
use bevy::render::camera::Projection;
use bevy::render::view::RenderLayers;
use glam::Vec2;
use tf_simulation::ship::{PlayerShip, ShipVelocity};

pub const WORLD_RENDER_LAYER: usize = 0;
pub const HUD_RENDER_LAYER: usize = 1;

const ISOMETRIC_YAW: f32 = -std::f32::consts::FRAC_PI_4;
const ISOMETRIC_PITCH: f32 = 0.615_479_7;
const CAMERA_DISTANCE: f32 = 6_000.0;

/// Marker for the main isometric camera.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct IsometricCamera;

/// Marker for the parent transform that follows the player.
#[derive(Component, Debug, Clone, Copy, Default)]
struct IsometricCameraRig;

/// Marker for the HUD camera (renders in screen-space).
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

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraFollowConfig>()
            .add_systems(Startup, (spawn_isometric_camera, spawn_hud_camera))
            .add_systems(
                PostUpdate,
                follow_player_ship.after(TransformSystem::TransformPropagate),
            );
    }
}

fn spawn_isometric_camera(mut commands: Commands, config: Res<CameraFollowConfig>) {
    let world_center = (config.world_min + config.world_max) * 0.5;

    let mut projection = OrthographicProjection::default_3d();
    projection.scale = config.zoom_scale;
    projection.near = -10_000.0;
    projection.far = 10_000.0;

    commands
        .spawn((
            Transform::from_translation(camera_translation_for_target(world_center)),
            IsometricCameraRig,
        ))
        .with_child((
            Camera3d::default(),
            Camera {
                order: 0,
                ..default()
            },
            RenderLayers::layer(WORLD_RENDER_LAYER),
            Projection::Orthographic(projection),
            local_camera_transform(),
            IsometricCamera,
        ));
}

fn spawn_hud_camera(mut commands: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = 1.0;
    projection.near = -2_000.0;
    projection.far = 2_000.0;

    // HUD camera is fixed in screen-space
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
    player_q: Query<(&GlobalTransform, Option<&ShipVelocity>), With<PlayerShip>>,
    mut rig_q: Query<&mut Transform, (With<IsometricCameraRig>, Without<PlayerShip>)>,
) {
    let Ok((player_global_tf, player_velocity)) = player_q.get_single() else {
        return;
    };

    let Ok(mut rig_tf) = rig_q.get_single_mut() else {
        return;
    };

    let player_world_pos = Vec2::new(
        player_global_tf.translation().x,
        player_global_tf.translation().y,
    );
    let lookahead = player_velocity
        .map(|velocity| velocity.linvel * config.lookahead_seconds)
        .unwrap_or(Vec2::ZERO);

    let world_target = clamp_world(
        player_world_pos + lookahead,
        config.world_min,
        config.world_max,
    );

    let blend = 1.0 - (-config.smoothing * time.delta_secs()).exp();
    let camera_target = camera_translation_for_target(world_target);
    rig_tf.translation = rig_tf.translation.lerp(camera_target, blend);
}

fn clamp_world(position: Vec2, min: Vec2, max: Vec2) -> Vec2 {
    position.clamp(min, max)
}

fn local_camera_transform() -> Transform {
    Transform::from_translation(camera_offset()).looking_at(Vec3::ZERO, Vec3::Z)
}

fn camera_translation_for_target(target: Vec2) -> Vec3 {
    world_to_render_translation(target, 0.0)
}

fn camera_offset() -> Vec3 {
    Vec3::new(
        ISOMETRIC_YAW.sin() * ISOMETRIC_PITCH.cos(),
        ISOMETRIC_YAW.cos() * ISOMETRIC_PITCH.cos(),
        ISOMETRIC_PITCH.sin(),
    ) * CAMERA_DISTANCE
}

fn world_to_render_translation(world: Vec2, elevation: f32) -> Vec3 {
    Vec3::new(world.x, world.y, elevation)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(left: Vec2, right: Vec2) {
        assert!((left.x - right.x).abs() < 1e-5);
        assert!((left.y - right.y).abs() < 1e-5);
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

    #[test]
    fn maps_world_xy_onto_render_xz() {
        let translated = world_to_render_translation(Vec2::new(12.0, 34.0), 5.0);
        assert_eq!(translated, Vec3::new(12.0, 34.0, 5.0));
    }

    #[test]
    fn camera_offset_pushes_camera_upward() {
        let offset = camera_offset();
        assert!(offset.z > 0.0);
    }

    #[test]
    fn camera_translation_tracks_world_target() {
        let translation = camera_translation_for_target(Vec2::new(100.0, 200.0));
        let expected = world_to_render_translation(Vec2::new(100.0, 200.0), 0.0);
        assert_eq!(translation, expected);
    }

    #[test]
    fn local_camera_transform_uses_fixed_offset() {
        let transform = local_camera_transform();
        assert_eq!(transform.translation, camera_offset());
    }
}
