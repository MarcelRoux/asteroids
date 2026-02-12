use macroquad::prelude::{Color, Vec2, vec2, screen_height, screen_width};
use macroquad::rand::gen_range;
use std::f32::consts::PI;

#[derive(Clone, Copy)]
pub(super) enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    pub(super) fn radius(&self) -> f32 {
        match self {
            AsteroidSize::Large => 28.0,
            AsteroidSize::Medium => 18.0,
            AsteroidSize::Small => 10.0,
        }
    }

    pub(super) fn next(&self) -> Option<AsteroidSize> {
        match self {
            AsteroidSize::Large => Some(AsteroidSize::Medium),
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Small => None,
        }
    }

    pub(super) fn score(&self) -> u32 {
        match self {
            AsteroidSize::Large => super::ASTEROID_SCORE_BASE,
            AsteroidSize::Medium => super::ASTEROID_SCORE_BASE * 2,
            AsteroidSize::Small => super::ASTEROID_SCORE_BASE * 4,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum HitTarget {
    LargeAsteroid,
    MediumAsteroid,
    SmallAsteroid,
    LargeAlien,
    SmallAlien,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BulletSource {
    Player,
    Alien,
}

#[derive(Clone, Copy)]
pub(super) enum AlienSize {
    Small,
    Large,
}

impl AlienSize {
    pub(super) fn y(&self) -> f32 {
        match self {
            AlienSize::Small => super::SMALL_ALIEN_Y,
            AlienSize::Large => super::LARGE_ALIEN_Y,
        }
    }

    pub(super) fn speed(&self) -> f32 {
        match self {
            AlienSize::Small => super::SMALL_ALIEN_SPEED,
            AlienSize::Large => super::LARGE_ALIEN_SPEED,
        }
    }

    pub(super) fn fire_interval(&self) -> f32 {
        match self {
            AlienSize::Small => super::SMALL_ALIEN_FIRE_INTERVAL,
            AlienSize::Large => super::LARGE_ALIEN_FIRE_INTERVAL,
        }
    }

    pub(super) fn cone_half_angle(&self, score: u32) -> f32 {
        let normalized = (score.min(100_000) as f32) / 100_000.0;
        let base = match self {
            AlienSize::Small => 0.9,
            AlienSize::Large => 1.1,
        };
        let tight = match self {
            AlienSize::Small => 0.25,
            AlienSize::Large => 0.35,
        };
        base - (base - tight) * normalized
    }

    pub(super) fn hit_radius(&self) -> f32 {
        match self {
            AlienSize::Small => 12.0,
            AlienSize::Large => 16.0,
        }
    }

    pub(super) fn score_value(&self) -> u32 {
        match self {
            AlienSize::Small => super::SMALL_ALIEN_SCORE,
            AlienSize::Large => super::LARGE_ALIEN_SCORE,
        }
    }

    pub(super) fn hit_target(&self) -> HitTarget {
        match self {
            AlienSize::Small => HitTarget::SmallAlien,
            AlienSize::Large => HitTarget::LargeAlien,
        }
    }
}

#[derive(Clone)]
pub(super) struct Alien {
    pub(super) position: Vec2,
    pub(super) velocity: Vec2,
    pub(super) size: AlienSize,
    pub(super) fire_timer: f32,
    pub(super) angle: f32,
}

impl Alien {
    pub(super) fn new(size: AlienSize, direction: f32, starting_x: f32, width: f32) -> Self {
        let y = size.y();
        let angle = if direction >= 0.0 { 0.0 } else { PI };
        Self {
            position: Vec2::new(starting_x.clamp(30.0, width - 30.0), y),
            velocity: Vec2::new(direction * size.speed(), 0.0),
            size,
            fire_timer: size.fire_interval(),
            angle,
        }
    }

    pub(super) fn update(&mut self, dt: f32, width: f32) {
        self.position.x += self.velocity.x * dt;
        if self.position.x < 30.0 {
            self.position.x = 30.0;
            self.velocity.x = -self.velocity.x;
        } else if self.position.x > width - 30.0 {
            self.position.x = width - 30.0;
            self.velocity.x = -self.velocity.x;
        }

        self.fire_timer -= dt;
        self.angle = if self.velocity.x >= 0.0 { 0.0 } else { PI };
    }

    pub(super) fn ready_to_fire(&self) -> bool {
        self.fire_timer <= 0.0
    }

    pub(super) fn reset_fire_timer(&mut self) {
        self.fire_timer = self.size.fire_interval();
    }

    pub(super) fn fire_direction(&self, ship_pos: Vec2, score: u32) -> Vec2 {
        let base = (ship_pos - self.position).to_angle();
        let cone = self.size.cone_half_angle(score);
        let offset = gen_range(-cone, cone);
        Vec2::from_angle(base + offset).normalize_or_zero()
    }
}

pub(super) struct Ship {
    pub(super) position: Vec2,
    pub(super) velocity: Vec2,
    pub(super) angle: f32,
}

impl Ship {
    pub(super) fn centered() -> Self {
        Self {
            position: vec2(screen_width() / 2.0, screen_height() / 2.0),
            velocity: Vec2::ZERO,
            angle: -PI / 2.0,
        }
    }
}

#[derive(Clone)]
pub(super) struct Asteroid {
    pub(super) position: Vec2,
    pub(super) velocity: Vec2,
    pub(super) size: AsteroidSize,
    pub(super) angle: f32,
    pub(super) rotation_speed: f32,
    pub(super) shape: Vec<Vec2>,
}

impl Asteroid {
    pub(super) fn new(size: AsteroidSize, position: Vec2, velocity: Vec2) -> Self {
        Self {
            position,
            velocity,
            size,
            angle: gen_range(0.0, 2.0 * PI),
            rotation_speed: gen_range(-0.8, 0.8),
            shape: generate_shape(size),
        }
    }

    pub(super) fn radius(&self) -> f32 {
        self.size.radius()
    }

    pub(super) fn points(&self) -> Vec<Vec2> {
        self.shape
            .iter()
            .map(|vertex| rotate_vector(*vertex, self.angle) + self.position)
            .collect()
    }

    pub(super) fn split(&self) -> Vec<Asteroid> {
        if let Some(next_size) = self.size.next() {
            let mut fragments = Vec::with_capacity(2);
            let base_len = self.velocity.length().max(super::ASTEROID_MIN_SPEED);
            let base_angle = self.velocity.to_angle();
            for i in 0..2 {
                let offset = Vec2::from_angle(base_angle + (i as f32 - 0.5) * 0.6);
                let velocity = offset * base_len;
                fragments.push(Asteroid::new(next_size, self.position, velocity));
            }
            fragments
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone)]
pub(super) struct Bullet {
    pub(super) position: Vec2,
    pub(super) velocity: Vec2,
    pub(super) ttl: f32,
    pub(super) source: BulletSource,
}

impl Bullet {
    pub(super) fn new(position: Vec2, velocity: Vec2, source: BulletSource) -> Self {
        Self {
            position,
            velocity,
            ttl: super::BULLET_TTL,
            source,
        }
    }
}

#[derive(Clone)]
pub(super) struct Debris {
    pub(super) position: Vec2,
    pub(super) velocity: Vec2,
    pub(super) ttl: f32,
    pub(super) color: Color,
}

impl Debris {
    pub(super) fn new(position: Vec2, velocity: Vec2, color: Color) -> Self {
        Self {
            position,
            velocity,
            ttl: super::DEBRIS_TTL,
            color,
        }
    }
}

pub(super) fn wrap_position(position: Vec2) -> Vec2 {
    let width = screen_width();
    let height = screen_height();
    let mut result = position;
    if result.x < 0.0 {
        result.x += width;
    } else if result.x > width {
        result.x -= width;
    }

    if result.y < 0.0 {
        result.y += height;
    } else if result.y > height {
        result.y -= height;
    }

    result
}

pub(super) fn clamp_length(value: Vec2, max: f32) -> Vec2 {
    let len_sq = value.length_squared();
    if len_sq > max * max {
        value.normalize() * max
    } else {
        value
    }
}

fn generate_shape(size: AsteroidSize) -> Vec<Vec2> {
    let base_radius = size.radius();
    let vertex_count = match size {
        AsteroidSize::Large => 12,
        AsteroidSize::Medium => 10,
        AsteroidSize::Small => 8,
    };
    (0..vertex_count)
        .map(|i| {
            let theta = (i as f32 / vertex_count as f32) * 2.0 * PI;
            let jitter = gen_range(0.8, 1.2);
            Vec2::from_angle(theta) * base_radius * jitter
        })
        .collect()
}

fn rotate_vector(vec: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(vec.x * cos - vec.y * sin, vec.x * sin + vec.y * cos)
}
