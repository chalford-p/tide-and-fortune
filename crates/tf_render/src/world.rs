use bevy::prelude::*;
use tf_simulation::ship::PlayerShip;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct IsometricRotationRoot;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct IsometricRoot;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct YSort;

#[derive(Component, Debug, Clone, Copy)]
struct OceanTile {
    phase: f32,
}

#[derive(Component, Debug, Clone, Copy, Default)]
struct ShipRotationFrame {
    index: u8,
}

/// World-space settings for the placeholder M1 render scene.
#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldRenderConfig {
    pub world_min: Vec2,
    pub world_max: Vec2,
    pub tile_size: f32,
}

impl Default for WorldRenderConfig {
    fn default() -> Self {
        Self {
            world_min: Vec2::ZERO,
            world_max: Vec2::splat(10_000.0),
            tile_size: 256.0,
        }
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldRenderConfig>()
            .add_systems(Startup, setup_world)
            .add_systems(
                Update,
                (
                    attach_player_ship_to_isometric_root,
                    animate_ocean_tiles,
                    y_sort_world_entities,
                    update_ship_sprite_frame,
                ),
            );
    }
}

fn setup_world(mut commands: Commands, config: Res<WorldRenderConfig>) {
    let world_root = commands
        .spawn((
            // Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
            Transform::default(),
            Visibility::default(),
            IsometricRoot,
        ))
        .id();

    // let isometric_root = commands.entity(world_root).with_child((
    //     // Transform::from_scale(Vec3::new(std::f32::consts::SQRT_2, std::f32::consts::SQRT_2 * 0.5, 1.0)),
    //     Visibility::default(),
    //     IsometricRoot,
    // ))
    // .id();

    let width = config.world_max.x - config.world_min.x;
    let height = config.world_max.y - config.world_min.y;
    let cols = (width / config.tile_size).ceil() as i32;
    let rows = (height / config.tile_size).ceil() as i32;

    commands.entity(world_root).with_children(|parent| {
        for y in 0..rows {
            for x in 0..cols {
                let world_x = config.world_min.x + x as f32 * config.tile_size + config.tile_size * 0.5;
                let world_y = config.world_min.y + y as f32 * config.tile_size + config.tile_size * 0.5;
                let phase = ((x + y) as f32 * 0.37).fract();

                parent.spawn((
                    Sprite {
                        color: Color::srgb(0.11, 0.38, 0.70),
                        custom_size: Some(Vec2::splat(config.tile_size)),
                        ..default()
                        },
                    Transform::from_xyz(world_x, world_y, 0.0),
                    OceanTile { phase },
                    YSort,
                ));
            }
        }

        parent.spawn((
            Sprite {
                    color: Color::srgb(0.23, 0.50, 0.22),
                    custom_size: Some(Vec2::new(620.0, 500.0)),
                    ..default()
                },
            Transform::from_xyz(6_400.0, 3_100.0, 0.0),
            YSort,
        ));
    });
}

fn attach_player_ship_to_isometric_root(
    mut commands: Commands,
    root_q: Query<Entity, With<IsometricRoot>>,
    mut ship_q: Query<
        (Entity, &mut Sprite),
        (
            With<PlayerShip>,
            Without<IsometricRoot>,
            Without<Parent>,
            Without<ShipRotationFrame>,
        ),
    >,
) {
    let Ok(root) = root_q.get_single() else {
        return;
    };

    for (entity, mut sprite) in &mut ship_q {
        // Set the ship's local transform to z=10 to ensure it's above ocean tiles
        commands.entity(root).add_child(entity);
        commands.entity(entity).insert((YSort, ShipRotationFrame::default()));
        // Set z-order high
        let mut transform = Transform::from_xyz(0.0, 0.0, 10.0);
        commands.entity(entity).insert(transform);

        sprite.color = Color::srgb(0.95, 0.72, 0.19);
        sprite.custom_size = Some(Vec2::new(54.0, 30.0));
    }
}

fn animate_ocean_tiles(time: Res<Time>, mut tiles: Query<(&OceanTile, &mut Sprite)>) {
    let t = time.elapsed_secs();

    for (tile, mut sprite) in &mut tiles {
        let wobble = (t * 0.75 + tile.phase * std::f32::consts::TAU).sin() * 0.08;
        let lightness = (0.42 + wobble).clamp(0.2, 0.9);
        sprite.color = Color::hsl(202.0 + wobble * 25.0, 0.62, lightness);
    }
}

fn y_sort_world_entities(mut renderables: Query<&mut Transform, (With<YSort>, Without<IsometricRoot>)>) {
    for mut transform in &mut renderables {
        transform.translation.z = -transform.translation.y * 0.001;
    }
}

fn update_ship_sprite_frame(
    mut ships: Query<(&mut Transform, &mut ShipRotationFrame), With<PlayerShip>>,
) {
    for (mut transform, mut frame) in &mut ships {
        let (_, _, heading) = transform.rotation.to_euler(EulerRot::XYZ);
        let wrapped = heading.rem_euclid(std::f32::consts::TAU);
        let frame_index = ((wrapped / std::f32::consts::TAU) * 16.0).round() as i32 % 16;
        frame.index = frame_index as u8;

        let snapped_heading = (f32::from(frame.index) / 16.0) * std::f32::consts::TAU;
        transform.rotation = Quat::from_rotation_z(snapped_heading);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_index_stays_in_16_frame_range() {
        let heading = std::f32::consts::TAU * 0.95;
        let frame_index = ((heading / std::f32::consts::TAU) * 16.0).round() as i32 % 16;
        assert!((0..16).contains(&frame_index));
    }
}
