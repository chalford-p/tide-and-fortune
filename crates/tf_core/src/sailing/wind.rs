use glam::Vec2;
use noise::{NoiseFn, Perlin};

/// Sea-state force classes derived from average wind speed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Beaufort {
    Calm,
    LightAir,
    LightBreeze,
    GentleBreeze,
    ModerateBreeze,
    FreshBreeze,
    StrongBreeze,
    NearGale,
    Gale,
    StrongGale,
    Storm,
    ViolentStorm,
    Hurricane,
}

impl Beaufort {
    pub fn from_speed(speed_mps: f32) -> Self {
        match speed_mps {
            s if s < 0.3 => Self::Calm,
            s if s < 1.6 => Self::LightAir,
            s if s < 3.4 => Self::LightBreeze,
            s if s < 5.5 => Self::GentleBreeze,
            s if s < 8.0 => Self::ModerateBreeze,
            s if s < 10.8 => Self::FreshBreeze,
            s if s < 13.9 => Self::StrongBreeze,
            s if s < 17.2 => Self::NearGale,
            s if s < 20.8 => Self::Gale,
            s if s < 24.5 => Self::StrongGale,
            s if s < 28.5 => Self::Storm,
            s if s < 32.7 => Self::ViolentStorm,
            _ => Self::Hurricane,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WindFieldConfig {
    pub world_min: Vec2,
    pub world_max: Vec2,
    pub cell_size: f32,
    pub min_speed: f32,
    pub max_speed: f32,
    pub gust_strength: f32,
}

impl Default for WindFieldConfig {
    fn default() -> Self {
        Self {
            world_min: Vec2::ZERO,
            world_max: Vec2::new(1_000.0, 1_000.0),
            cell_size: 100.0,
            min_speed: 2.0,
            max_speed: 14.0,
            gust_strength: 0.3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WindField {
    config: WindFieldConfig,
    nx: usize,
    ny: usize,
    cell_width: f32,
    cell_height: f32,
    cells: Vec<Vec2>,
    drift_noise: Perlin,
    gust_noise: Perlin,
}

impl WindField {
    pub fn new(config: WindFieldConfig) -> Self {
        assert!(config.world_max.x > config.world_min.x, "invalid world x bounds");
        assert!(config.world_max.y > config.world_min.y, "invalid world y bounds");
        assert!(config.cell_size > 0.0, "cell_size must be > 0");
        assert!(config.min_speed >= 0.0, "min_speed must be >= 0");
        assert!(config.max_speed >= config.min_speed, "max_speed must be >= min_speed");

        let world_size = config.world_max - config.world_min;
        let nx = (world_size.x / config.cell_size).ceil() as usize + 1;
        let ny = (world_size.y / config.cell_size).ceil() as usize + 1;

        let cell_width = world_size.x / (nx.saturating_sub(1).max(1) as f32);
        let cell_height = world_size.y / (ny.saturating_sub(1).max(1) as f32);

        let base_speed = (config.min_speed + config.max_speed) * 0.5;
        let cells = vec![Vec2::new(base_speed, 0.0); nx * ny];

        Self {
            config,
            nx,
            ny,
            cell_width,
            cell_height,
            cells,
            drift_noise: Perlin::new(11),
            gust_noise: Perlin::new(97),
        }
    }

    pub fn config(&self) -> WindFieldConfig {
        self.config
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.nx, self.ny)
    }

    /// Returns the apparent wind at `pos` for a vessel moving with `ship_velocity`.
    /// Apparent wind = true wind − ship velocity.
    pub fn apparent_wind_at(&self, pos: Vec2, ship_velocity: Vec2) -> Vec2 {
        self.at(pos) - ship_velocity
    }

    pub fn at(&self, pos: Vec2) -> Vec2 {
        let clamped = pos.clamp(self.config.world_min, self.config.world_max);
        let rel = clamped - self.config.world_min;

        let x = (rel.x / self.cell_width).clamp(0.0, (self.nx - 1) as f32);
        let y = (rel.y / self.cell_height).clamp(0.0, (self.ny - 1) as f32);

        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x0 + 1).min(self.nx - 1);
        let y1 = (y0 + 1).min(self.ny - 1);

        let tx = x - x0 as f32;
        let ty = y - y0 as f32;

        let c00 = self.cell(x0, y0);
        let c10 = self.cell(x1, y0);
        let c01 = self.cell(x0, y1);
        let c11 = self.cell(x1, y1);

        let top = c00.lerp(c10, tx);
        let bottom = c01.lerp(c11, tx);
        top.lerp(bottom, ty)
    }

    pub fn update(&mut self, time: f32) {
        let t = time as f64;
        let drift_dir = self.drift_noise.get([t * 0.02, 0.0]) as f32;
        let drift_speed = self.drift_noise.get([0.0, t * 0.01]) as f32;

        let angle = drift_dir * std::f32::consts::PI;
        let mut direction = Vec2::new(angle.cos(), angle.sin());
        if direction.length_squared() < f32::EPSILON {
            direction = Vec2::X;
        }

        let target_speed = remap_unit(
            drift_speed,
            self.config.min_speed,
            self.config.max_speed,
        );

        for y in 0..self.ny {
            for x in 0..self.nx {
                let idx = self.index(x, y);
                let px = x as f64;
                let py = y as f64;

                // Higher frequency temporal noise keeps gusts short-lived.
                let gust_scalar = self.gust_noise.get([px * 0.15, py * 0.15, t * 0.8]) as f32;
                let gust_angle = self.gust_noise.get([px * 0.15 + 31.0, py * 0.15 - 17.0, t * 0.8])
                    as f32
                    * std::f32::consts::TAU;
                let gust_dir = Vec2::new(gust_angle.cos(), gust_angle.sin());
                let gust_mag = gust_scalar * self.config.gust_strength * target_speed;

                let mut wind = direction * target_speed + gust_dir * gust_mag;
                let speed = wind.length().clamp(self.config.min_speed, self.config.max_speed);
                if wind.length_squared() < f32::EPSILON {
                    wind = direction * speed;
                } else {
                    wind = wind.normalize() * speed;
                }

                self.cells[idx] = wind;
            }
        }
    }

    pub fn set_constant(&mut self, wind: Vec2) {
        for cell in &mut self.cells {
            *cell = wind;
        }
    }

    pub fn average_magnitude(&self) -> f32 {
        let sum: f32 = self.cells.iter().map(|v| v.length()).sum();
        sum / self.cells.len() as f32
    }

    pub fn beaufort(&self) -> Beaufort {
        Beaufort::from_speed(self.average_magnitude())
    }

    fn cell(&self, x: usize, y: usize) -> Vec2 {
        self.cells[self.index(x, y)]
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.nx + x
    }
}

fn remap_unit(value: f32, min: f32, max: f32) -> f32 {
    let t = ((value + 1.0) * 0.5).clamp(0.0, 1.0);
    min + (max - min) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_field() -> WindField {
        let mut field = WindField::new(WindFieldConfig {
            world_min: Vec2::ZERO,
            world_max: Vec2::new(100.0, 100.0),
            cell_size: 100.0,
            min_speed: 0.0,
            max_speed: 40.0,
            gust_strength: 0.0,
        });

        let i00 = field.index(0, 0);
        let i10 = field.index(1, 0);
        let i01 = field.index(0, 1);
        let i11 = field.index(1, 1);

        field.cells[i00] = Vec2::new(0.0, 0.0);
        field.cells[i10] = Vec2::new(10.0, 0.0);
        field.cells[i01] = Vec2::new(20.0, 0.0);
        field.cells[i11] = Vec2::new(30.0, 0.0);
        field
    }

    #[test]
    fn bilinear_interpolation_blends_neighbor_cells() {
        let field = test_field();
        let sampled = field.at(Vec2::new(50.0, 50.0));
        assert!((sampled.x - 15.0).abs() < 1e-5);
        assert!(sampled.y.abs() < 1e-5);
    }

    #[test]
    fn out_of_bounds_positions_are_clamped() {
        let field = test_field();

        let below = field.at(Vec2::new(-50.0, -20.0));
        assert_eq!(below, Vec2::new(0.0, 0.0));

        let above = field.at(Vec2::new(200.0, 250.0));
        assert_eq!(above, Vec2::new(30.0, 0.0));
    }

    #[test]
    fn update_keeps_cell_speeds_within_bounds() {
        let mut field = WindField::new(WindFieldConfig {
            world_min: Vec2::new(-500.0, -500.0),
            world_max: Vec2::new(500.0, 500.0),
            cell_size: 100.0,
            min_speed: 3.0,
            max_speed: 12.0,
            gust_strength: 0.5,
        });

        for i in 0..120 {
            field.update(i as f32 * 0.5);
            for speed in field.cells.iter().map(|v| v.length()) {
                assert!(speed >= 3.0 - 1e-5);
                assert!(speed <= 12.0 + 1e-5);
            }
        }
    }
}