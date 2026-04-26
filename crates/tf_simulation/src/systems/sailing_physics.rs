use bevy_ecs::prelude::{Query, Res};
use bevy_math::{EulerRot, Quat, Vec3};
use bevy_time::Time;
use bevy_transform::components::Transform;
use glam::Vec2;
use tf_core::sailing::points_of_sail::PointOfSail;

use crate::ship::{Helm, SailAssistTier, SailState, Ship, ShipVelocity};
use crate::{GameMode, WindFieldResource};

const RUDDER_RESPONSE_RATE: f32 = 2.8;
const MAX_TURN_RATE_RAD_PER_SEC: f32 = 0.65;
const HEADING_SNAP_RAD: f32 = 30.0_f32.to_radians();
const GYBE_SPEED_PENALTY: f32 = 0.60;
const LEEWAY_COEFFICIENT: f32 = 0.16;

pub fn sailing_physics_system(
    time: Res<Time>,
    mode: Res<GameMode>,
    wind_field: Res<WindFieldResource>,
    mut ships: Query<(&Ship, &mut Helm, &SailState, &mut ShipVelocity, &mut Transform)>,
) {
    if *mode != GameMode::Sailing {
        return;
    }

    let dt = time.delta_secs();
    if dt <= f32::EPSILON {
        return;
    }

    for (ship, mut helm, sail_state, mut velocity, mut transform) in &mut ships {
        let heading = transform.rotation.to_euler(EulerRot::XYZ).2;
        let ship_forward_bevy = transform.rotation * Vec3::X;
        let mut ship_forward = Vec2::new(ship_forward_bevy.x, ship_forward_bevy.y);
        if ship_forward.length_squared() < f32::EPSILON {
            ship_forward = Vec2::X;
        } else {
            ship_forward = ship_forward.normalize();
        }

        let ship_position = Vec2::new(transform.translation.x, transform.translation.y);
        let true_wind = wind_field.field.at(ship_position);

        let apparent_wind = true_wind - velocity.linvel;
        if apparent_wind.length_squared() < f32::EPSILON {
            transform.translation.x += velocity.linvel.x * dt;
            transform.translation.y += velocity.linvel.y * dt;
            continue;
        }

        let (point_of_sail, _) = PointOfSail::from_vectors(apparent_wind, ship_forward);
        let trim_efficiency = tier_trim_efficiency(sail_state, point_of_sail);

        let drive_force = compute_drive_force(ship, apparent_wind, ship_forward, trim_efficiency, point_of_sail);
        let leeway_accel = compute_leeway_accel(ship, apparent_wind, ship_forward);

        // Integrate velocity with simple Euler integration.
        let drive_accel = ship_forward * (drive_force / ship.displacement_tonnes.max(1.0));
        velocity.linvel += (drive_accel + leeway_accel) * dt;

        let heading_delta = wrap_angle(helm.target_heading - heading);
        helm.rudder_angle = (heading_delta / std::f32::consts::FRAC_PI_2).clamp(-1.0, 1.0);

        let turn_step = (helm.rudder_angle * RUDDER_RESPONSE_RATE * dt)
            .clamp(-MAX_TURN_RATE_RAD_PER_SEC * dt, MAX_TURN_RATE_RAD_PER_SEC * dt);

        let mut new_heading = heading + turn_step;
        let turning_toward_wind = is_turning_toward_wind(heading, heading_delta, apparent_wind);

        if sail_state.tier == SailAssistTier::Tier1
            && point_of_sail.gybe_risk(turning_toward_wind)
            && helm.rudder_angle.abs() > 0.1
        {
            let snap_sign = heading_delta.signum();
            if snap_sign != 0.0 {
                new_heading += snap_sign * HEADING_SNAP_RAD;
                velocity.linvel *= GYBE_SPEED_PENALTY;
                helm.target_heading = new_heading;
            }
        }

        velocity.angvel = turn_step / dt;

        transform.rotation = Quat::from_rotation_z(new_heading);
        transform.translation.x += velocity.linvel.x * dt;
        transform.translation.y += velocity.linvel.y * dt;
    }
}

fn tier_trim_efficiency(sail_state: &SailState, point_of_sail: PointOfSail) -> f32 {
    match sail_state.tier {
        SailAssistTier::Tier1 => point_of_sail.auto_trim_efficiency(),
        SailAssistTier::Tier2 => 0.9,
        SailAssistTier::Tier3 => 1.0,
    }
}

fn compute_drive_force(
    ship: &Ship,
    apparent_wind: Vec2,
    ship_forward: Vec2,
    trim_efficiency: f32,
    point_of_sail: PointOfSail,
) -> f32 {
    if point_of_sail == PointOfSail::InIrons {
        return 0.0;
    }

    let wind_speed = apparent_wind.length();
    let alignment = apparent_wind.normalize().dot(ship_forward).abs().clamp(0.0, 1.0);
    let cosine_drive = (1.0 - alignment * 0.5).clamp(0.25, 1.0);
    let hull_factor = ship.ship_type.base_drive_coefficient();

    wind_speed * wind_speed * trim_efficiency * cosine_drive * hull_factor * 0.09
}

fn compute_leeway_accel(ship: &Ship, apparent_wind: Vec2, ship_forward: Vec2) -> Vec2 {
    let starboard = Vec2::new(-ship_forward.y, ship_forward.x);
    let side_force = apparent_wind.dot(starboard);
    let resistance = ship.ship_type.hull_resistance().max(1.0);

    // Drift to leeward: opposite side of wind pressure.
    let leeway = -starboard * (side_force / resistance) * LEEWAY_COEFFICIENT;
    if leeway.length_squared() <= 1e-8 {
        Vec2::ZERO
    } else {
        leeway
    }
}

fn is_turning_toward_wind(current_heading: f32, heading_delta: f32, apparent_wind: Vec2) -> bool {
    if heading_delta.abs() <= f32::EPSILON || apparent_wind.length_squared() <= f32::EPSILON {
        return false;
    }

    let wind_heading = apparent_wind.y.atan2(apparent_wind.x);
    let current_offset = wrap_angle(wind_heading - current_heading).abs();
    let trial_heading = current_heading + heading_delta.signum() * 5.0_f32.to_radians();
    let trial_offset = wrap_angle(wind_heading - trial_heading).abs();

    trial_offset < current_offset
}

fn wrap_angle(angle: f32) -> f32 {
    let mut wrapped = angle;
    while wrapped > std::f32::consts::PI {
        wrapped -= std::f32::consts::TAU;
    }
    while wrapped < -std::f32::consts::PI {
        wrapped += std::f32::consts::TAU;
    }
    wrapped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ship::{SailPlan, ShipType};

    #[test]
    fn drive_force_is_zero_in_irons() {
        let ship = Ship {
            ship_type: ShipType::Sloop,
            displacement_tonnes: 100.0,
            sail_plan: SailPlan::ForeAndAft,
        };

        let force = compute_drive_force(
            &ship,
            Vec2::new(8.0, 0.0),
            Vec2::new(1.0, 0.0),
            1.0,
            PointOfSail::InIrons,
        );

        assert_eq!(force, 0.0);
    }

    #[test]
    fn wrap_angle_normalizes_to_pi_interval() {
        let wrapped = wrap_angle(4.5);
        assert!(wrapped <= std::f32::consts::PI);
        assert!(wrapped >= -std::f32::consts::PI);
    }
}
