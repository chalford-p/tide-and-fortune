use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use tf_simulation::ship::PlayerShip;

use crate::camera::WORLD_RENDER_LAYER;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct IsometricRotationRoot;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct IsometricRoot;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct YSort;

#[derive(Component, Debug, Clone, Copy, Default)]
struct PlayerShipMesh;

#[derive(Resource, Debug, Clone)]
struct OceanMaterialHandles {
    even: Handle<StandardMaterial>,
    odd: Handle<StandardMaterial>,
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
                    attach_player_ship_visual,
                    animate_ocean_tiles,
                ),
            );
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<WorldRenderConfig>,
) {
    let world_root = commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            IsometricRoot,
        ))
        .id();

    let width = config.world_max.x - config.world_min.x;
    let height = config.world_max.y - config.world_min.y;
    let cols = (width / config.tile_size).ceil() as i32;
    let rows = (height / config.tile_size).ceil() as i32;
    let ocean_mesh = meshes.add(Plane3d::new(Vec3::Z, Vec2::splat(config.tile_size * 0.5)).mesh());
    let ocean_material_even = materials.add(StandardMaterial {
        base_color: ocean_tile_color(0.0, 0.0),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        unlit: true,
        ..default()
    });
    let ocean_material_odd = materials.add(StandardMaterial {
        base_color: ocean_tile_color(0.5, 0.0),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        unlit: true,
        ..default()
    });
    let island_mesh = meshes.add(Cuboid::new(620.0, 500.0, 24.0));
    let island_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.23, 0.50, 0.22),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        unlit: true,
        ..default()
    });
    commands.insert_resource(OceanMaterialHandles {
        even: ocean_material_even.clone(),
        odd: ocean_material_odd.clone(),
    });

    commands.entity(world_root).with_children(|parent| {
        for y in 0..rows {
            for x in 0..cols {
                let world_x = config.world_min.x + x as f32 * config.tile_size + config.tile_size * 0.5;
                let world_y = config.world_min.y + y as f32 * config.tile_size + config.tile_size * 0.5;
                let material = if (x + y) % 2 == 0 {
                    ocean_material_even.clone()
                } else {
                    ocean_material_odd.clone()
                };

                parent.spawn((
                    Mesh3d(ocean_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(world_x, world_y, -0.5),
                    RenderLayers::layer(WORLD_RENDER_LAYER),
                ));
            }
        }

        parent.spawn((
            Mesh3d(island_mesh),
            MeshMaterial3d(island_material),
            Transform::from_xyz(6_400.0, 3_100.0, 12.0),
            RenderLayers::layer(WORLD_RENDER_LAYER),
        ));
    });
}

fn attach_player_ship_visual(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ship_q: Query<Entity, (With<PlayerShip>, Added<PlayerShip>)>,
) {
    let base_triangle = Triangle2d::new(Vec2::new(-27.0, -15.0), Vec2::new(27.0, 0.0), Vec2::new(-27.0, 15.0));

    let ship_mesh = meshes.add(Extrusion::new(base_triangle, 10.0));

    for entity in &ship_q {
        let ship_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.72, 0.19),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            unlit: true,
            ..default()
        });

        commands.entity(entity).insert((
            PlayerShipMesh,
            Mesh3d(ship_mesh.clone()),
            MeshMaterial3d(ship_material),
            Transform::from_xyz(0.0, 0.0, 6.0),
            RenderLayers::layer(WORLD_RENDER_LAYER),
        ));
    }
}

fn animate_ocean_tiles(
    time: Res<Time>,
    ocean_materials: Res<OceanMaterialHandles>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();

    if let Some(material) = materials.get_mut(&ocean_materials.even) {
        material.base_color = ocean_tile_color(0.0, t);
    }

    if let Some(material) = materials.get_mut(&ocean_materials.odd) {
        material.base_color = ocean_tile_color(0.5, t);
    }
}

fn ocean_tile_color(phase: f32, time: f32) -> Color {
    let wobble = (time * 0.75 + phase * std::f32::consts::TAU).sin() * 0.08;
    let lightness = (0.42 + wobble).clamp(0.2, 0.9);
    Color::hsl(202.0 + wobble * 25.0, 0.62, lightness)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocean_tile_color_stays_visible() {
        let color = ocean_tile_color(0.37, 2.0).to_srgba();
        assert!(color.red > 0.0);
        assert!(color.green > 0.0);
        assert!(color.blue > 0.0);
    }
}
