use crate::camera::ISOMETRIC_YAW;
use bevy::prelude::*;
use glam::Vec2 as CoreVec2;
use tf_core::sailing::points_of_sail::PointOfSail;
use tf_core::sailing::wind::Beaufort;
use tf_simulation::ship::{PlayerShip, ShipVelocity};
use tf_simulation::WindFieldResource;

#[derive(Component, Debug, Clone, Copy, Default)]
struct HudRoot;

#[derive(Component, Debug, Clone, Copy, Default)]
struct ApparentWindNeedle;

#[derive(Component, Debug, Clone, Copy, Default)]
struct TrueWindNeedle;

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
        app.add_systems(Startup, setup_hud)
            .add_systems(Update, (update_wind_hud, update_point_of_sail_indicator));
    }
}

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let apparent_color = Color::srgb(0.93, 0.82, 0.28);
    let true_color = Color::srgb(0.31, 0.80, 0.94);
    let needle_image: Handle<Image> = asset_server.load("hud/needle.png");
    let gauge_radius = 27.0_f32;

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                left: Val::Px(16.0),
                width: Val::Px(350.0),
                min_height: Val::Px(170.0),
                padding: UiRect::all(Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.06, 0.11, 0.82)),
            BorderRadius::all(Val::Px(6.0)),
            HudRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("WIND HUD"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.95, 0.98)),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|top_row| {
                    spawn_wind_gauge(
                        top_row,
                        "APPARENT",
                        gauge_radius,
                        apparent_color,
                        ApparentWindNeedle,
                        needle_image.clone(),
                    );
                    spawn_wind_gauge(
                        top_row,
                        "TRUE",
                        gauge_radius,
                        true_color,
                        TrueWindNeedle,
                        needle_image.clone(),
                    );
                    top_row
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|sail_parent| {
                            sail_parent.spawn((
                                Text::new("POINT OF SAIL"),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.85, 0.89, 0.95)),
                            ));
                            spawn_point_of_sail_list(sail_parent);
                        });
                });

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|beaufort_parent| {
                    beaufort_parent.spawn((
                        Text::new("BEAUFORT"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.85, 0.89, 0.95)),
                    ));

                    beaufort_parent
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(4.0),
                            align_items: AlignItems::Stretch,
                            ..default()
                        })
                        .with_children(|bar_parent| {
                            for index in 0..13_u8 {
                                bar_parent.spawn((
                                    Node {
                                        flex_grow: 1.0,
                                        flex_basis: Val::Px(0.0),
                                        min_width: Val::Px(8.0),
                                        height: Val::Px(10.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.22, 0.25, 0.29)),
                                    BorderRadius::all(Val::Px(2.0)),
                                    BeaufortBar { level: index },
                                ));
                            }
                        });
                });

        });
}

fn spawn_wind_gauge<M: Component>(
    parent: &mut ChildBuilder,
    label: &'static str,
    radius: f32,
    color: Color,
    marker: M,
    needle_image: Handle<Image>,
) {
    let diam = radius * 2.0;
    // The needle image is sized to almost fill the gauge circle.
    let needle_size = diam - 4.0;
    let needle_half = needle_size * 0.5;

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|gauge_parent| {
            gauge_parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(color),
            ));

            gauge_parent
                .spawn((
                    Node {
                        width: Val::Px(diam),
                        height: Val::Px(diam),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.11, 0.14, 0.18, 0.95)),
                    BorderRadius::all(Val::Px(radius)),
                ))
                .with_children(|circle| {
                    // Zero-size anchor at the gauge center. Its Transform.rotation is
                    // updated each frame to point at the wind direction. The actual
                    // image is a child offset by -half so it visually rotates around
                    // the center while remaining inside the UI clipping tree.
                    circle
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(radius),
                                top: Val::Px(radius),
                                width: Val::Px(0.0),
                                height: Val::Px(0.0),
                                ..default()
                            },
                            Transform::default(),
                            marker,
                        ))
                        .with_children(|anchor| {
                            anchor.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(-needle_half),
                                    top: Val::Px(-needle_half),
                                    width: Val::Px(needle_size),
                                    height: Val::Px(needle_size),
                                    ..default()
                                },
                                ImageNode {
                                    image: needle_image,
                                    color,
                                    ..default()
                                },
                            ));
                        });
                });
        });
}

fn spawn_point_of_sail_list(parent: &mut ChildBuilder) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|list| {
            for zone in PointOfSail::ORDERED {
                list.spawn((
                    Node {
                        max_width: Val::Px(80.0),
                        height: Val::Px(16.0),
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(0.0)),
                        ..default()
                    },
                    BackgroundColor(zone_color(zone).with_alpha(0.25)),
                    BorderRadius::all(Val::Px(3.0)),
                    PointOfSailSegment { zone },
                ));
            }
        });
}

fn update_wind_hud(
    wind_field: Res<WindFieldResource>,
    ship_q: Query<(&Transform, Option<&ShipVelocity>), With<PlayerShip>>,
    mut needle_q: Query<
        (
            &mut Transform,
            Option<&ApparentWindNeedle>,
            Option<&TrueWindNeedle>,
        ),
        (
            Without<PlayerShip>,
            Or<(With<ApparentWindNeedle>, With<TrueWindNeedle>)>,
        ),
    >,
    mut bars_q: Query<(&BeaufortBar, &mut BackgroundColor), Without<PlayerShip>>,
) {
    let Ok((ship_tf, velocity)) = ship_q.get_single() else {
        return;
    };

    let sample_pos = CoreVec2::new(ship_tf.translation.x, ship_tf.translation.y);
    let true_wind = wind_field.field.at(sample_pos);
    let apparent_wind = true_wind - velocity.map(|v| v.linvel).unwrap_or(CoreVec2::ZERO);

    let apparent_angle = apparent_wind.y.atan2(apparent_wind.x) - ISOMETRIC_YAW;
    let true_angle = true_wind.y.atan2(true_wind.x) - ISOMETRIC_YAW;

    for (mut needle_tf, apparent, true_wind_needle) in &mut needle_q {
        if apparent.is_some() {
            needle_tf.rotation = if apparent_wind.length_squared() <= f32::EPSILON {
                Quat::IDENTITY
            } else {
                Quat::from_rotation_z(apparent_angle)
            };
        }

        if true_wind_needle.is_some() {
            needle_tf.rotation = if true_wind.length_squared() <= f32::EPSILON {
                Quat::IDENTITY
            } else {
                Quat::from_rotation_z(true_angle)
            };
        }
    }

    let level = beaufort_to_level(wind_field.field.beaufort());
    for (bar, mut bar_color) in &mut bars_q {
        bar_color.0 = if bar.level <= level {
            Color::srgb(0.92, 0.80, 0.28)
        } else {
            Color::srgb(0.22, 0.25, 0.29)
        };
    }
}

fn update_point_of_sail_indicator(
    wind_field: Res<WindFieldResource>,
    ship_q: Query<(&Transform, Option<&ShipVelocity>), With<PlayerShip>>,
    mut segments: Query<(&PointOfSailSegment, &mut BackgroundColor), Without<PlayerShip>>,
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

    for (segment, mut segment_color) in &mut segments {
        let zone_color = zone_color(segment.zone);
        segment_color.0 = if segment.zone == zone {
            zone_color
        } else {
            zone_color.with_alpha(0.25)
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
