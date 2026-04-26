use bevy_ecs::entity::Entity;
use bevy_ecs::event::{Event, EventWriter};
use bevy_ecs::prelude::{Query, Res, With};
use bevy_input::keyboard::KeyCode;
use bevy_input::ButtonInput;

use crate::ship::{Helm, PlayerShip};
use crate::GameMode;

const DIRECTION_SNAP_STEP: f32 = std::f32::consts::FRAC_PI_4;
const FINE_ADJUST_STEP: f32 = DIRECTION_SNAP_STEP / 4.0;

#[derive(Event, Debug, Clone, Copy)]
pub struct HeadingChanged {
    pub entity: Entity,
    pub target_heading: f32,
}

pub fn player_input_system(
    mode: Res<GameMode>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ships: Query<(Entity, &mut Helm), With<PlayerShip>>,
    mut heading_changed: EventWriter<HeadingChanged>,
) {
    if *mode != GameMode::Sailing {
        return;
    }

    let x = axis_positive(
        &keyboard,
        [KeyCode::KeyD, KeyCode::ArrowRight],
        [KeyCode::KeyA, KeyCode::ArrowLeft],
    );
    let y = axis_positive(
        &keyboard,
        [KeyCode::KeyW, KeyCode::ArrowUp],
        [KeyCode::KeyS, KeyCode::ArrowDown],
    );

    if x == 0 && y == 0 {
        return;
    }

    let direction_bearing = direction_to_bearing(x, y);
    let fine_adjust = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    for (entity, mut helm) in &mut ships {
        let next_heading = if fine_adjust {
            let delta = shortest_delta(helm.target_heading, direction_bearing);
            let step = delta.clamp(-FINE_ADJUST_STEP, FINE_ADJUST_STEP);
            normalize_bearing(helm.target_heading + step)
        } else {
            direction_bearing
        };

        if (next_heading - helm.target_heading).abs() <= f32::EPSILON {
            continue;
        }

        helm.target_heading = next_heading;
        heading_changed.send(HeadingChanged {
            entity,
            target_heading: next_heading,
        });
    }
}

fn axis_positive(
    keyboard: &ButtonInput<KeyCode>,
    positive: [KeyCode; 2],
    negative: [KeyCode; 2],
) -> i8 {
    let pos = keyboard.any_pressed(positive) as i8;
    let neg = keyboard.any_pressed(negative) as i8;
    pos - neg
}

fn direction_to_bearing(x: i8, y: i8) -> f32 {
    let raw = (x as f32).atan2(y as f32);
    normalize_bearing(raw)
}

fn normalize_bearing(angle: f32) -> f32 {
    let mut wrapped = angle % std::f32::consts::TAU;
    if wrapped < 0.0 {
        wrapped += std::f32::consts::TAU;
    }
    wrapped
}

fn shortest_delta(current: f32, target: f32) -> f32 {
    let mut delta = (target - current) % std::f32::consts::TAU;
    if delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    } else if delta < -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }
    delta
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::event::Events;
    use bevy_ecs::prelude::{IntoSystem, World};
    use bevy_ecs::system::System;

    fn run_once(world: &mut World) {
        let mut system = IntoSystem::into_system(player_input_system);
        system.initialize(world);
        system.run((), world);
        system.apply_deferred(world);
    }

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(GameMode::Sailing);
        world.insert_resource(ButtonInput::<KeyCode>::default());
        world.insert_resource(Events::<HeadingChanged>::default());
        world.spawn((
            PlayerShip,
            Helm {
                target_heading: 0.0,
                rudder_angle: 0.0,
            },
        ));
        world
    }

    fn approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-5, "expected {a} ~= {b}");
    }

    #[test]
    fn maps_wasd_and_arrows_to_world_bearing() {
        let mut world = setup_world();
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);

        run_once(&mut world);

        let mut query = world.query::<&Helm>();
        let helm = query.single(&world);
        approx_eq(helm.target_heading, std::f32::consts::FRAC_PI_2);

        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::KeyD);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowUp);

        run_once(&mut world);

        let helm = query.single(&world);
        approx_eq(helm.target_heading, 0.0);
    }

    #[test]
    fn sets_diagonal_heading() {
        let mut world = setup_world();
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyA);

        run_once(&mut world);

        let mut query = world.query::<&Helm>();
        let helm = query.single(&world);
        approx_eq(helm.target_heading, std::f32::consts::TAU * 7.0 / 8.0);
    }

    #[test]
    fn preserves_heading_when_no_input() {
        let mut world = setup_world();

        {
            let mut query = world.query::<&mut Helm>();
            let mut helm = query.single_mut(&mut world);
            helm.target_heading = std::f32::consts::PI;
        }

        run_once(&mut world);

        let mut query = world.query::<&Helm>();
        let helm = query.single(&world);
        approx_eq(helm.target_heading, std::f32::consts::PI);
    }

    #[test]
    fn applies_fine_adjust_step_when_shift_is_held() {
        let mut world = setup_world();
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ShiftLeft);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);

        run_once(&mut world);

        let mut query = world.query::<&Helm>();
        let helm = query.single(&world);
        approx_eq(helm.target_heading, FINE_ADJUST_STEP);
    }

    #[test]
    fn runs_only_in_sailing_mode() {
        let mut world = setup_world();
        *world.resource_mut::<GameMode>() = GameMode::Harbor;
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyS);

        run_once(&mut world);

        let mut query = world.query::<&Helm>();
        let helm = query.single(&world);
        approx_eq(helm.target_heading, 0.0);
    }

    #[test]
    fn emits_heading_changed_event() {
        let mut world = setup_world();
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyS);

        run_once(&mut world);

        world.resource_mut::<Events<HeadingChanged>>().update();
        let collected: Vec<HeadingChanged> = world
            .resource_mut::<Events<HeadingChanged>>()
            .drain()
            .collect();

        assert_eq!(collected.len(), 1);
        approx_eq(collected[0].target_heading, std::f32::consts::PI);
    }
}
