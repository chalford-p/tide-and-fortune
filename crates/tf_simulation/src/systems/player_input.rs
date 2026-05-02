use bevy_ecs::prelude::{Query, Res, With};
use bevy_log::error_once;
use bevy_input::keyboard::KeyCode;
use bevy_input::ButtonInput;

use crate::ship::{Helm, PlayerShip, Rudder};
use crate::GameMode;

pub fn player_input_system(
    mode: Res<GameMode>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ships: Query<(&mut Helm, &Rudder), With<PlayerShip>>,
) {
    if *mode != GameMode::Sailing {
        return;
    }

    // Positive = D/Right, negative = A/Left.
    let turn = axis_signed(
        &keyboard,
        [KeyCode::KeyD, KeyCode::ArrowRight],
        [KeyCode::KeyA, KeyCode::ArrowLeft],
    );

    let fine_adjust = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let rudder_rate: f32 = if fine_adjust { 0.25 } else { 1.0 };

    if let Ok((mut helm, rudder)) = ships.get_single_mut() {
        // Positive turn (D) = CW = negative rotation in Bevy's CCW-positive space.
        helm.rudder_angle = if turn == 0 {
            0.0
        } else {
            -(turn as f32) * rudder.max_angle * rudder_rate
        };
    } else {
        error_once!(
            "Player input system found {} PlayerShip entities; expected exactly 1",
            ships.iter().count()
        );
    }
}

fn axis_signed(
    keyboard: &ButtonInput<KeyCode>,
    positive: [KeyCode; 2],
    negative: [KeyCode; 2],
) -> i8 {
    let pos = keyboard.any_pressed(positive) as i8;
    let neg = keyboard.any_pressed(negative) as i8;
    pos - neg
}


#[cfg(test)]
mod tests {
    use super::*;
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
        world.spawn((
            PlayerShip,
            Helm { rudder_angle: 0.0 },
            Rudder { max_angle: std::f32::consts::FRAC_PI_4 },
        ));
        world
    }

    fn approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-5, "expected {a} ~= {b}");
    }

    #[test]
    fn right_key_sets_full_negative_rudder() {
        let mut world = setup_world();
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);

        run_once(&mut world);

        let mut query = world.query::<&Helm>();
        let helm = query.single(&world);
        approx_eq(helm.rudder_angle, -std::f32::consts::FRAC_PI_4);
    }

    #[test]
    fn left_key_sets_full_positive_rudder() {
        let mut world = setup_world();
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyA);

        run_once(&mut world);

        let mut query = world.query::<&Helm>();
        let helm = query.single(&world);
        approx_eq(helm.rudder_angle, std::f32::consts::FRAC_PI_4);
    }

    #[test]
    fn fine_adjust_uses_quarter_max_angle() {
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
        approx_eq(helm.rudder_angle, -std::f32::consts::FRAC_PI_4 * 0.25);
    }

    #[test]
    fn no_input_returns_rudder_to_zero() {
        let mut world = setup_world();

        {
            let mut query = world.query::<&mut Helm>();
            query.single_mut(&mut world).rudder_angle = std::f32::consts::FRAC_PI_4;
        }

        run_once(&mut world);

        let mut query = world.query::<&Helm>();
        approx_eq(query.single(&world).rudder_angle, 0.0);
    }

    #[test]
    fn runs_only_in_sailing_mode() {
        let mut world = setup_world();
        *world.resource_mut::<GameMode>() = GameMode::Harbor;
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);

        run_once(&mut world);

        let mut query = world.query::<&Helm>();
        approx_eq(query.single(&world).rudder_angle, 0.0);
    }

    #[test]
    fn arrow_keys_work_like_wasd() {
        let mut world = setup_world();
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowLeft);

        run_once(&mut world);

        let mut query = world.query::<&Helm>();
        approx_eq(query.single(&world).rudder_angle, std::f32::consts::FRAC_PI_4);
    }
}

