use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use glam::Vec2 as CoreVec2;
use tf_core::sailing::points_of_sail::PointOfSail;
use tf_core::sailing::wind::Beaufort;
use tf_simulation::ship::{PlayerShip, ShipVelocity};
use tf_simulation::WindFieldResource;

use crate::camera::HUD_RENDER_LAYER;

#[derive(Component, Debug, Clone, Copy, Default)]
struct HudRoot;

#[derive(Component, Debug, Clone, Copy, Default)]
struct WindNeedle;

#[derive(Component, Debug, Clone, Copy)]
struct BeaufortBar {
    level: u8,
}

#[derive(Component, Debug, Clone, Copy)]
struct PointOfSailSegment {
    zone: PointOfSail,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud).add_systems(
            Update,
            (
                update_wind_hud,
                update_point_of_sail_indicator,
            ),
        );
    }
}

fn setup_hud(mut commands: Commands) {

    let root = commands
        .spawn((
            Transform::from_xyz(-420.0, 230.0, 2_000.0),
            Visibility::default(),
            RenderLayers::layer(HUD_RENDER_LAYER),
            HudRoot,
        ))
        .id();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Sprite {
                color: Color::srgba(0.03, 0.06, 0.11, 0.82),
                custom_size: Some(Vec2::new(280.0, 140.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::default(),
            RenderLayers::layer(HUD_RENDER_LAYER),
        ));

        parent.spawn((
            Sprite {
                color: Color::srgb(0.85, 0.87, 0.90),
                custom_size: Some(Vec2::new(8.0, 62.0)),
                ..default()
            },
            Transform::from_xyz(-82.0, 12.0, 1.0),
            Visibility::default(),
            RenderLayers::layer(HUD_RENDER_LAYER),
            WindNeedle,
        ));

        for index in 0..13_u8 {
            parent.spawn((
                Sprite {
                    color: Color::srgb(0.22, 0.25, 0.29),
                    custom_size: Some(Vec2::new(8.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(-40.0 + f32::from(index) * 12.0, -48.0, 1.0),
                Visibility::default(),
                RenderLayers::layer(HUD_RENDER_LAYER),
                BeaufortBar { level: index },
            ));
        }

        spawn_point_of_sail_arc(parent);
    });
}

fn spawn_point_of_sail_arc(parent: &mut ChildBuilder) {
    let zones = [
        PointOfSail::InIrons,
        PointOfSail::CloseHauled,
        PointOfSail::CloseReach,
        PointOfSail::BeamReach,
        PointOfSail::BroadReach,
        PointOfSail::Running,
    ];

    for (index, zone) in zones.into_iter().enumerate() {
        let angle = std::f32::consts::PI * (index as f32 / (zones.len() - 1) as f32);
        let x = 72.0 + angle.cos() * 34.0;
        let y = 2.0 + angle.sin() * 34.0;

        parent.spawn((
            Sprite {
                color: zone_color(zone).with_alpha(0.25),
                custom_size: Some(Vec2::new(15.0, 15.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 1.0),
            Visibility::default(),
            RenderLayers::layer(HUD_RENDER_LAYER),
            PointOfSailSegment { zone },
        ));
    }
}

fn update_wind_hud(
    wind_field: Res<WindFieldResource>,
    ship_q: Query<(&Transform, Option<&ShipVelocity>), With<PlayerShip>>,
    mut needle_q: Query<&mut Transform, (With<WindNeedle>, Without<PlayerShip>)>,
    mut bars_q: Query<(&BeaufortBar, &mut Sprite), Without<PlayerShip>>,
) {
    let Ok((ship_tf, velocity)) = ship_q.get_single() else {
        return;
    };

    let sample_pos = CoreVec2::new(ship_tf.translation.x, ship_tf.translation.y);
    let true_wind = wind_field.field.at(sample_pos);
    let apparent_wind = true_wind - velocity.map(|v| v.linvel).unwrap_or(CoreVec2::ZERO);

    let angle = apparent_wind.y.atan2(apparent_wind.x) - std::f32::consts::FRAC_PI_2;
    if let Ok(mut needle_tf) = needle_q.get_single_mut() {
        needle_tf.rotation = Quat::from_rotation_z(angle);
    }

    let level = beaufort_to_level(wind_field.field.beaufort());
    for (bar, mut sprite) in &mut bars_q {
        sprite.color = if bar.level <= level {
            Color::srgb(0.92, 0.80, 0.28)
        } else {
            Color::srgb(0.22, 0.25, 0.29)
        };
    }
}

fn update_point_of_sail_indicator(
    wind_field: Res<WindFieldResource>,
    ship_q: Query<(&Transform, Option<&ShipVelocity>), With<PlayerShip>>,
    mut segments: Query<(&PointOfSailSegment, &mut Sprite), Without<PlayerShip>>,
) {
    let Ok((ship_tf, velocity)) = ship_q.get_single() else {
        return;
    };

    let sample_pos = CoreVec2::new(ship_tf.translation.x, ship_tf.translation.y);
    let true_wind = wind_field.field.at(sample_pos);
    let apparent_wind = true_wind - velocity.map(|v| v.linvel).unwrap_or(CoreVec2::ZERO);

    if apparent_wind.length_squared() <= f32::EPSILON {
        return;
    }

    let ship_forward = (ship_tf.rotation * Vec3::X).truncate();
    if ship_forward.length_squared() <= f32::EPSILON {
        return;
    }

    let ship_forward_core = CoreVec2::new(ship_forward.x, ship_forward.y).normalize();
    let (zone, _) = PointOfSail::from_vectors(apparent_wind, ship_forward_core);

    for (segment, mut sprite) in &mut segments {
        let color = zone_color(segment.zone);
        sprite.color = if segment.zone == zone {
            color
        } else {
            color.with_alpha(0.25)
        };
    }
}

fn zone_color(zone: PointOfSail) -> Color {
    match zone {
        PointOfSail::InIrons => Color::srgb(0.84, 0.17, 0.20),
        PointOfSail::CloseHauled => Color::srgb(0.88, 0.45, 0.16),
        PointOfSail::CloseReach => Color::srgb(0.93, 0.75, 0.20),
        PointOfSail::BeamReach => Color::srgb(0.34, 0.80, 0.31),
        PointOfSail::BroadReach => Color::srgb(0.13, 0.68, 0.80),
        PointOfSail::Running => Color::srgb(0.35, 0.45, 0.86),
    }
}

fn beaufort_to_level(scale: Beaufort) -> u8 {
    match scale {
        Beaufort::Calm => 0,
        Beaufort::LightAir => 1,
        Beaufort::LightBreeze => 2,
        Beaufort::GentleBreeze => 3,
        Beaufort::ModerateBreeze => 4,
        Beaufort::FreshBreeze => 5,
        Beaufort::StrongBreeze => 6,
        Beaufort::NearGale => 7,
        Beaufort::Gale => 8,
        Beaufort::StrongGale => 9,
        Beaufort::Storm => 10,
        Beaufort::ViolentStorm => 11,
        Beaufort::Hurricane => 12,
    }
}
