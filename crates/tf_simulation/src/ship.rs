use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_sprite::Sprite;
use bevy_transform::components::Transform;
use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShipType {
    Sloop,
    Brig,
    Frigate,
}

impl ShipType {
    pub fn base_drive_coefficient(self) -> f32 {
        match self {
            ShipType::Sloop => 1.0,
            ShipType::Brig => 0.85,
            ShipType::Frigate => 0.75,
        }
    }

    pub fn hull_resistance(self) -> f32 {
        match self {
            ShipType::Sloop => 2_400.0,
            ShipType::Brig => 3_200.0,
            ShipType::Frigate => 4_200.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SailPlan {
    ForeAndAft,
    Mixed,
    SquareRig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SailAssistTier {
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SailSlot {
    Jib,
    Main,
    Spinnaker,
}

impl SailSlot {
    pub const COUNT: usize = 3;

    pub fn index(self) -> usize {
        match self {
            SailSlot::Jib => 0,
            SailSlot::Main => 1,
            SailSlot::Spinnaker => 2,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Ship {
    pub ship_type: ShipType,
    pub displacement_tonnes: f32,
    pub sail_plan: SailPlan,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Helm {
    pub target_heading: f32,
    pub rudder_angle: f32,
}

#[derive(Component, Debug, Clone)]
pub struct SailState {
    pub tier: SailAssistTier,
    pub active_sails: [bool; SailSlot::COUNT],
    pub trim_per_sail: [f32; SailSlot::COUNT],
}

impl SailState {
    pub fn with_all_sails(tier: SailAssistTier) -> Self {
        Self {
            tier,
            active_sails: [true; SailSlot::COUNT],
            trim_per_sail: [0.5; SailSlot::COUNT],
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ShipVelocity {
    pub linvel: Vec2,
    pub angvel: f32,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerShip;

#[derive(Bundle, Debug, Clone)]
pub struct ShipBundle {
    pub ship: Ship,
    pub helm: Helm,
    pub sail_state: SailState,
    pub velocity: ShipVelocity,
    pub player_ship: PlayerShip,
    pub transform: Transform,
    pub sprite: Sprite,
}

impl Default for ShipBundle {
    fn default() -> Self {
        Self {
            ship: Ship {
                ship_type: ShipType::Sloop,
                displacement_tonnes: 120.0,
                sail_plan: SailPlan::ForeAndAft,
            },
            helm: Helm {
                target_heading: 0.0,
                rudder_angle: 0.0,
            },
            sail_state: SailState::with_all_sails(SailAssistTier::Tier1),
            velocity: ShipVelocity::default(),
            player_ship: PlayerShip,
            transform: Transform::default(),
            sprite: Sprite::default(),
        }
    }
}
