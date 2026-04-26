use glam::Vec2;

/// Represents the six points of sail, classified by angle to wind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointOfSail {
    /// 0–30° off wind, both sides. Hardest to sail.
    InIrons,
    /// 30–60° off wind.
    CloseHauled,
    /// 60–80° off wind.
    CloseReach,
    /// 80–100° off wind. Typically most efficient.
    BeamReach,
    /// 100–150° off wind.
    BroadReach,
    /// 150–180° off wind. Running directly downwind.
    Running,
}

impl PointOfSail {
    /// Given apparent wind vector and ship forward vector,
    /// return current point of sail and angle in degrees.
    ///
    /// The angle is measured from the ship's forward direction to the wind direction.
    pub fn from_vectors(apparent_wind: Vec2, ship_forward: Vec2) -> (PointOfSail, f32) {
        // Normalize vectors
        let wind_norm = apparent_wind.normalize();
        let forward_norm = ship_forward.normalize();

        // Calculate angle between vectors using dot product
        // This gives us the acute angle (0-180°)
        let cos_angle = wind_norm.dot(forward_norm).clamp(-1.0, 1.0);
        let angle_rad = cos_angle.acos();
        let angle_deg = angle_rad.to_degrees();

        // Classify by angle
        let point_of_sail = if angle_deg < 30.0 {
            PointOfSail::InIrons
        } else if angle_deg < 60.0 {
            PointOfSail::CloseHauled
        } else if angle_deg < 80.0 {
            PointOfSail::CloseReach
        } else if angle_deg < 100.0 {
            PointOfSail::BeamReach
        } else if angle_deg < 150.0 {
            PointOfSail::BroadReach
        } else {
            PointOfSail::Running
        };

        (point_of_sail, angle_deg)
    }

    /// Auto-trim efficiency for Tier 1 sailing (0.0–1.0).
    /// Represents how well auto-selected sails perform at this angle.
    pub fn auto_trim_efficiency(&self) -> f32 {
        match self {
            PointOfSail::InIrons => 0.1,      // Very poor efficiency
            PointOfSail::CloseHauled => 0.6,  // Moderate efficiency
            PointOfSail::CloseReach => 0.8,   // Good efficiency
            PointOfSail::BeamReach => 1.0,    // Best efficiency
            PointOfSail::BroadReach => 0.9,   // Nearly optimal
            PointOfSail::Running => 0.7,      // Good downwind efficiency
        }
    }

    /// Continuous gybe risk score (0.0–1.0) for UI and gameplay tuning.
    ///
    /// Risk is non-zero only when running and turning in the risky direction.
    /// The score scales linearly with turn rate and saturates at 1.0.
    pub fn gybe_risk_score(&self, turning_toward_wind: bool, turn_rate_deg_per_sec: f32) -> f32 {
        // Tune this threshold to set when a turn should be considered maximally risky.
        const DANGEROUS_TURN_RATE_DEG_PER_SEC: f32 = 20.0;

        if *self != PointOfSail::Running || !turning_toward_wind {
            return 0.0;
        }

        (turn_rate_deg_per_sec.max(0.0) / DANGEROUS_TURN_RATE_DEG_PER_SEC).clamp(0.0, 1.0)
    }

    /// True if a gybe warning should fire (broad off, turning downwind)
    pub fn gybe_risk(&self, turning_toward_wind: bool) -> bool {
        // Gybe risk only applies when running and turning further away from wind
        // (i.e., turning toward the downwind direction)
        *self == PointOfSail::Running && turning_toward_wind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
        (a - b).abs() < epsilon
    }

    #[test]
    fn test_in_irons_zone() {
        // Wind straight ahead (0°)
        let wind = Vec2::new(1.0, 0.0);
        let forward = Vec2::new(1.0, 0.0);
        let (point, angle) = PointOfSail::from_vectors(wind, forward);
        assert_eq!(point, PointOfSail::InIrons);
        assert!(approx_eq(angle, 0.0, 1.0));

        // Wind at 15° off bow
        let wind = Vec2::new(1.0, 0.3).normalize();
        let forward = Vec2::new(1.0, 0.0);
        let (point, angle) = PointOfSail::from_vectors(wind, forward);
        assert_eq!(point, PointOfSail::InIrons);
        assert!(angle < 30.0);
    }

    #[test]
    fn test_close_hauled_zone() {
        // Wind at 45° off bow (in close hauled range)
        let wind = Vec2::new(1.0, 1.0).normalize(); // 45°
        let forward = Vec2::new(1.0, 0.0);
        let (point, angle) = PointOfSail::from_vectors(wind, forward);
        assert_eq!(point, PointOfSail::CloseHauled);
        assert!(approx_eq(angle, 45.0, 2.0));
    }

    #[test]
    fn test_close_reach_zone() {
        // Wind at 70° off bow
        let wind = Vec2::new(1.0, 2.5).normalize();
        let forward = Vec2::new(1.0, 0.0);
        let (point, angle) = PointOfSail::from_vectors(wind, forward);
        assert_eq!(point, PointOfSail::CloseReach);
        assert!(angle > 60.0 && angle < 80.0);
    }

    #[test]
    fn test_beam_reach_zone() {
        // Wind at 90° off beam (perfect beam reach)
        let wind = Vec2::new(0.0, 1.0);
        let forward = Vec2::new(1.0, 0.0);
        let (point, angle) = PointOfSail::from_vectors(wind, forward);
        assert_eq!(point, PointOfSail::BeamReach);
        assert!(approx_eq(angle, 90.0, 1.0));
    }

    #[test]
    fn test_broad_reach_zone() {
        // Wind at 120° off bow (broad reach)
        let wind = Vec2::new(-1.0, 2.0).normalize();
        let forward = Vec2::new(1.0, 0.0);
        let (point, angle) = PointOfSail::from_vectors(wind, forward);
        assert_eq!(point, PointOfSail::BroadReach);
        assert!(angle > 100.0 && angle < 150.0);
    }

    #[test]
    fn test_running_zone() {
        // Wind from behind (straight downwind, 180°)
        let wind = Vec2::new(-1.0, 0.0);
        let forward = Vec2::new(1.0, 0.0);
        let (point, angle) = PointOfSail::from_vectors(wind, forward);
        assert_eq!(point, PointOfSail::Running);
        assert!(approx_eq(angle, 180.0, 1.0));

        // Wind at 160° off bow (still running)
        let wind = Vec2::new(-1.0, -0.4).normalize();
        let forward = Vec2::new(1.0, 0.0);
        let (point, angle) = PointOfSail::from_vectors(wind, forward);
        assert_eq!(point, PointOfSail::Running);
        assert!(angle > 150.0);
    }

    #[test]
    fn test_boundary_angles() {
        let forward = Vec2::new(1.0, 0.0);

        // Test angles near boundaries - verify they calculate correctly
        // and that classifications are appropriate for angles near boundaries
        
        // Near 30° boundary
        let wind_25 = Vec2::new(0.906, 0.423).normalize(); // ~25°
        let (point_25, angle_25) = PointOfSail::from_vectors(wind_25, forward);
        assert!(angle_25 > 20.0 && angle_25 < 30.0);
        assert_eq!(point_25, PointOfSail::InIrons);

        let wind_35 = Vec2::new(0.819, 0.574).normalize(); // ~35°
        let (point_35, angle_35) = PointOfSail::from_vectors(wind_35, forward);
        assert!(angle_35 > 30.0 && angle_35 < 40.0);
        assert_eq!(point_35, PointOfSail::CloseHauled);

        // Near 60° boundary
        let wind_55 = Vec2::new(0.574, 0.819).normalize(); // ~55°
        let (point_55, angle_55) = PointOfSail::from_vectors(wind_55, forward);
        assert!(angle_55 > 50.0 && angle_55 < 60.0);
        assert_eq!(point_55, PointOfSail::CloseHauled);

        let wind_65 = Vec2::new(0.423, 0.906).normalize(); // ~65°
        let (point_65, angle_65) = PointOfSail::from_vectors(wind_65, forward);
        assert!(angle_65 > 60.0 && angle_65 < 70.0);
        assert_eq!(point_65, PointOfSail::CloseReach);

        // Near 80° boundary
        let wind_75 = Vec2::new(0.259, 0.966).normalize(); // ~75°
        let (point_75, angle_75) = PointOfSail::from_vectors(wind_75, forward);
        assert!(angle_75 > 70.0 && angle_75 < 80.0);
        assert_eq!(point_75, PointOfSail::CloseReach);

        let wind_85 = Vec2::new(0.087, 0.996).normalize(); // ~85°
        let (point_85, angle_85) = PointOfSail::from_vectors(wind_85, forward);
        assert!(angle_85 > 80.0 && angle_85 < 90.0);
        assert_eq!(point_85, PointOfSail::BeamReach);

        // Near 100° boundary
        let wind_95 = Vec2::new(-0.087, 0.996).normalize(); // ~95°
        let (point_95, angle_95) = PointOfSail::from_vectors(wind_95, forward);
        assert!(angle_95 > 90.0 && angle_95 < 100.0);
        assert_eq!(point_95, PointOfSail::BeamReach);

        let wind_105 = Vec2::new(-0.259, 0.966).normalize(); // ~105°
        let (point_105, angle_105) = PointOfSail::from_vectors(wind_105, forward);
        assert!(angle_105 > 100.0 && angle_105 < 110.0);
        assert_eq!(point_105, PointOfSail::BroadReach);

        // Near 150° boundary
        let wind_145 = Vec2::new(-0.819, 0.574).normalize(); // ~145°
        let (point_145, angle_145) = PointOfSail::from_vectors(wind_145, forward);
        assert!(angle_145 > 140.0 && angle_145 < 150.0);
        assert_eq!(point_145, PointOfSail::BroadReach);

        let wind_155 = Vec2::new(-0.906, 0.423).normalize(); // ~155°
        let (point_155, angle_155) = PointOfSail::from_vectors(wind_155, forward);
        assert!(angle_155 > 150.0 && angle_155 < 160.0);
        assert_eq!(point_155, PointOfSail::Running);
    }

    #[test]
    fn test_auto_trim_efficiency() {
        assert_eq!(PointOfSail::InIrons.auto_trim_efficiency(), 0.1);
        assert_eq!(PointOfSail::CloseHauled.auto_trim_efficiency(), 0.6);
        assert_eq!(PointOfSail::CloseReach.auto_trim_efficiency(), 0.8);
        assert_eq!(PointOfSail::BeamReach.auto_trim_efficiency(), 1.0);
        assert_eq!(PointOfSail::BroadReach.auto_trim_efficiency(), 0.9);
        assert_eq!(PointOfSail::Running.auto_trim_efficiency(), 0.7);
    }

    #[test]
    fn test_gybe_risk_running_turning_downwind() {
        // Gybe risk should fire: Running and turning toward wind (away from current direction)
        assert!(PointOfSail::Running.gybe_risk(true));
    }

    #[test]
    fn test_gybe_risk_running_not_turning() {
        // No gybe risk: Running but not turning toward wind
        assert!(!PointOfSail::Running.gybe_risk(false));
    }

    #[test]
    fn test_gybe_risk_not_running() {
        // No gybe risk in other points of sail, even when turning toward wind
        assert!(!PointOfSail::InIrons.gybe_risk(true));
        assert!(!PointOfSail::CloseHauled.gybe_risk(true));
        assert!(!PointOfSail::CloseReach.gybe_risk(true));
        assert!(!PointOfSail::BeamReach.gybe_risk(true));
        assert!(!PointOfSail::BroadReach.gybe_risk(true));
    }

    #[test]
    fn test_gybe_risk_score_only_in_running_and_risky_direction() {
        assert!(approx_eq(
            PointOfSail::InIrons.gybe_risk_score(true, 20.0),
            0.0,
            1e-6
        ));
        assert!(approx_eq(
            PointOfSail::Running.gybe_risk_score(false, 20.0),
            0.0,
            1e-6
        ));
    }

    #[test]
    fn test_gybe_risk_score_scales_with_turn_rate() {
        assert!(approx_eq(
            PointOfSail::Running.gybe_risk_score(true, 0.0),
            0.0,
            1e-6
        ));
        assert!(approx_eq(
            PointOfSail::Running.gybe_risk_score(true, 10.0),
            0.5,
            1e-6
        ));
        assert!(approx_eq(
            PointOfSail::Running.gybe_risk_score(true, 20.0),
            1.0,
            1e-6
        ));
        assert!(approx_eq(
            PointOfSail::Running.gybe_risk_score(true, 40.0),
            1.0,
            1e-6
        ));
    }

    #[test]
    fn test_gybe_risk_score_clamps_negative_turn_rate() {
        assert!(approx_eq(
            PointOfSail::Running.gybe_risk_score(true, -5.0),
            0.0,
            1e-6
        ));
    }

    #[test]
    fn test_symmetrical_angles() {
        // Wind on starboard
        let wind_starboard = Vec2::new(0.707, 0.707).normalize();
        let forward = Vec2::new(1.0, 0.0);
        let (point_sb, angle_sb) = PointOfSail::from_vectors(wind_starboard, forward);

        // Wind on port (opposite y)
        let wind_port = Vec2::new(0.707, -0.707).normalize();
        let (point_pt, angle_pt) = PointOfSail::from_vectors(wind_port, forward);

        // Should classify the same and have same angle
        assert_eq!(point_sb, point_pt);
        assert!(approx_eq(angle_sb, angle_pt, 0.1));
    }
}
