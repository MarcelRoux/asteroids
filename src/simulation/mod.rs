use crate::ai::{AsteroidSnapshot, WorldSnapshot};
use crate::config::{GameConfig, PlayerControllerMode};
use crate::controllers::{ControlIntent, Controller};
use macroquad::prelude::{
    Color, LIGHTGRAY, Vec2, WHITE, draw_circle, draw_line, draw_triangle, screen_height,
    screen_width, vec2,
};
use macroquad::rand::gen_range;
use std::f32::consts::{FRAC_PI_2, PI};

const SHIP_THRUST: f32 = 400.0;
const SHIP_MAX_SPEED: f32 = 320.0;
const SHIP_DRAG: f32 = 1.5;
const SHIP_ROTATION_SPEED: f32 = 3.0;
const SHIP_SIZE: f32 = 14.0;
const INVULN_DURATION: f32 = 4.0;

const ASTEROID_MIN_SPEED: f32 = 30.0;
const ASTEROID_MAX_SPEED: f32 = 90.0;
const ASTEROID_SPAWN_INTERVAL: f32 = 0.50;
const BULLET_SPEED: f32 = 520.0;
const BULLET_RADIUS: f32 = 2.0;
const BULLET_TTL: f32 = 2.0;
const PRIMARY_FIRE_RATE: f32 = 5.0;
const SECONDARY_FIRE_RATE: f32 = PRIMARY_FIRE_RATE / 5.0;
const SECONDARY_SPREAD: f32 = PI / 12.0;
const SECONDARY_COUNT: usize = 5;
const MAX_LIVES: u32 = 3;
const ASTEROID_SCORE_BASE: u32 = 100;
const DEBRIS_TTL: f32 = 1.0;
const DEBRIS_SPEED: f32 = 120.0;
const DEBRIS_COUNT: usize = 6;
const DEBRIS_COLOR: Color = Color::new(1.0, 0.75, 0.3, 1.0);

#[derive(Clone, Copy)]
enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    fn radius(&self) -> f32 {
        match self {
            AsteroidSize::Large => 28.0,
            AsteroidSize::Medium => 18.0,
            AsteroidSize::Small => 10.0,
        }
    }

    fn next(&self) -> Option<AsteroidSize> {
        match self {
            AsteroidSize::Large => Some(AsteroidSize::Medium),
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Small => None,
        }
    }

    fn score(&self) -> u32 {
        match self {
            AsteroidSize::Large => ASTEROID_SCORE_BASE,
            AsteroidSize::Medium => ASTEROID_SCORE_BASE * 2,
            AsteroidSize::Small => ASTEROID_SCORE_BASE * 4,
        }
    }
}

pub struct Simulation {
    controller: Box<dyn Controller>,
    policy: SimulationPolicy,
    ship: Ship,
    asteroids: Vec<Asteroid>,
    spawn_acc: f32,
    primary_cooldown: f32,
    secondary_cooldown: f32,
    bullets: Vec<Bullet>,
    debris: Vec<Debris>,
    lives: u32,
    dt: f32,
    status: SimulationStatus,
    invuln_timer: f32,
}

impl Simulation {
    pub fn new(config: GameConfig) -> Self {
        let controller: Box<dyn Controller> = match &config.player_controller {
            PlayerControllerMode::Human => {
                Box::new(crate::controllers::human::HumanController::default())
            }
            PlayerControllerMode::Ai { profile } => {
                Box::new(crate::ai::AiController::new(profile.clone()))
            }
        };

        Self {
            controller,
            policy: SimulationPolicy::from_config(&config),
            ship: Ship::centered(),
            asteroids: Vec::new(),
            spawn_acc: 0.0,
            primary_cooldown: 0.0,
            secondary_cooldown: 0.0,
            bullets: Vec::new(),
            debris: Vec::new(),
            lives: MAX_LIVES,
            dt: 1.0 / 60.0,
            status: SimulationStatus::default(),
            invuln_timer: INVULN_DURATION,
        }
    }

    pub fn controller(&mut self) -> &mut dyn Controller {
        self.controller.as_mut()
    }

    pub fn snapshot(&self) -> WorldSnapshot {
        WorldSnapshot {
            ship_position: self.ship.position,
            ship_velocity: self.ship.velocity,
            ship_angle: self.ship.angle,
            asteroids: self
                .asteroids
                .iter()
                .map(|ast| AsteroidSnapshot {
                    position: ast.position,
                    velocity: ast.velocity,
                    radius: ast.radius(),
                })
                .collect(),
        }
    }

    pub fn dt(&self) -> f32 {
        self.dt
    }

    pub fn apply_intent(&mut self, intent: ControlIntent) {
        self.status.last_intent = Some(intent);
    }

    pub fn set_controller(&mut self, controller: Box<dyn Controller>) {
        self.controller = controller;
    }

    pub fn step(&mut self) {
        self.status.frame += 1;
        let intent = self.status.last_intent.unwrap_or_default();
        self.update_ship(intent);
        self.handle_firing(intent);
        self.update_asteroids();
        self.spawn_acc += self.dt;
        while self.spawn_acc >= ASTEROID_SPAWN_INTERVAL {
            self.spawn_acc -= ASTEROID_SPAWN_INTERVAL;
            self.spawn_asteroid();
        }
        self.update_bullets();
        self.update_debris();
        self.resolve_collisions();
        self.primary_cooldown = (self.primary_cooldown - self.dt).max(0.0);
        self.secondary_cooldown = (self.secondary_cooldown - self.dt).max(0.0);
        self.invuln_timer = (self.invuln_timer - self.dt).max(0.0);
        self.status.asteroid_count = self.asteroids.len();
        self.status.bullet_count = self.bullets.len();
        self.status.active_bodies = 1 + self.asteroids.len() + self.bullets.len();
        self.status.primary_cooldown = self.primary_cooldown;
        self.status.secondary_cooldown = self.secondary_cooldown;
        self.status.frame_time = self.dt;
        self.status.fps = 1.0 / self.dt;
        self.status.lives = self.lives;
        self.status.game_over = self.lives == 0;
    }

    pub fn policy(&mut self) -> &mut SimulationPolicy {
        &mut self.policy
    }

    pub fn draw_debug(&self) {
        for asteroid in &self.asteroids {
            let points = asteroid.points();
            if points.len() > 1 {
                for i in 0..points.len() {
                    let a = points[i];
                    let b = points[(i + 1) % points.len()];
                    draw_line(a.x, a.y, b.x, b.y, 2.0, LIGHTGRAY);
                }
            }
        }

        let (nose, left, right) = self.ship_triangle();
        draw_triangle(nose, left, right, WHITE);
        if self.invuln_timer > 0.0 {
            let alpha = ((self.invuln_timer / INVULN_DURATION) * 0.8).clamp(0.2, 0.8);
            draw_circle(
                self.ship.position.x,
                self.ship.position.y,
                SHIP_SIZE * 1.4,
                Color::new(0.2, 0.8, 1.0, alpha),
            );
        }

        for bullet in &self.bullets {
            draw_circle(
                bullet.position.x,
                bullet.position.y,
                BULLET_RADIUS,
                Color::new(1.0, 0.9, 0.4, 1.0),
            );
        }

        for debris in &self.debris {
            draw_circle(debris.position.x, debris.position.y, 2.0, DEBRIS_COLOR);
        }
    }

    pub fn status(&self) -> SimulationStatus {
        self.status.clone()
    }

    fn update_ship(&mut self, intent: ControlIntent) {
        self.ship.angle += intent.turn * SHIP_ROTATION_SPEED * self.dt;
        let forward = Vec2::from_angle(self.ship.angle);

        if intent.thrust > 0.0 {
            self.ship.velocity += forward * (intent.thrust * SHIP_THRUST * self.dt);
        }

        self.ship.velocity -= self.ship.velocity * SHIP_DRAG * self.dt;
        self.ship.velocity = clamp_length(self.ship.velocity, SHIP_MAX_SPEED);
        self.ship.position = wrap_position(self.ship.position + self.ship.velocity * self.dt);
    }

    fn handle_firing(&mut self, intent: ControlIntent) {
        if intent.fire_primary && self.primary_cooldown <= 0.0 {
            self.primary_cooldown = 1.0 / PRIMARY_FIRE_RATE;
            let forward = Vec2::from_angle(self.ship.angle);
            let spawn_pos = self.ship.position + forward * SHIP_SIZE;
            let spawn_velocity = forward * BULLET_SPEED;
            self.spawn_bullet(spawn_pos, spawn_velocity);
        }

        if intent.fire_secondary && self.secondary_cooldown <= 0.0 {
            self.secondary_cooldown = 1.0 / SECONDARY_FIRE_RATE;
            let base_angle = self.ship.angle;
            let center = (SECONDARY_COUNT as f32 - 1.0) * 0.5;
            for i in 0..SECONDARY_COUNT {
                let offset = (i as f32 - center) * SECONDARY_SPREAD;
                let dir = Vec2::from_angle(base_angle + offset);
                let spawn_pos = self.ship.position + dir * SHIP_SIZE;
                self.spawn_bullet(spawn_pos, dir * BULLET_SPEED);
            }
        }
    }

    fn update_bullets(&mut self) {
        self.bullets.retain_mut(|bullet| {
            bullet.ttl -= self.dt;
            if bullet.ttl <= 0.0 {
                return false;
            }
            bullet.position = wrap_position(bullet.position + bullet.velocity * self.dt);
            true
        });
    }

    fn update_debris(&mut self) {
        self.debris.retain_mut(|debris| {
            debris.ttl -= self.dt;
            if debris.ttl <= 0.0 {
                return false;
            }
            debris.position = wrap_position(debris.position + debris.velocity * self.dt);
            true
        });
    }

    fn resolve_collisions(&mut self) {
        let mut bullet_hits = vec![false; self.bullets.len()];
        let mut asteroid_hits = vec![false; self.asteroids.len()];
        let ship_radius = SHIP_SIZE * 0.9;
        let mut fragments = Vec::new();
        let mut earned_score: u32 = 0;
        let mut destroyed_asteroids = Vec::new();
        for (bi, bullet) in self.bullets.iter().enumerate() {
            if bullet_hits[bi] {
                continue;
            }
            for (ai, asteroid) in self.asteroids.iter().enumerate() {
                if asteroid_hits[ai] {
                    continue;
                }
                let radius_sum = asteroid.radius() + BULLET_RADIUS;
                if bullet.position.distance_squared(asteroid.position) <= radius_sum * radius_sum {
                    bullet_hits[bi] = true;
                    asteroid_hits[ai] = true;
                    earned_score = earned_score.saturating_add(asteroid.size.score());
                    fragments.extend(asteroid.split());
                    destroyed_asteroids.push(asteroid.clone());
                    break;
                }
            }
        }

        let mut ship_hit = false;
        for (ai, asteroid) in self.asteroids.iter().enumerate() {
            let radius_sum = asteroid.radius() + ship_radius;
            if self.invuln_timer <= 0.0
                && self.ship.position.distance_squared(asteroid.position) <= radius_sum * radius_sum
            {
                asteroid_hits[ai] = true;
                ship_hit = true;
            }
        }

        if ship_hit {
            if self.lives > 0 {
                self.lives -= 1;
            }
            if self.lives > 0 {
                self.reset_ship();
            }
        }

        self.status.score = self.status.score.saturating_add(earned_score);

        for asteroid in destroyed_asteroids {
            self.spawn_debris(&asteroid);
        }

        let mut survivors = Vec::new();
        for (i, asteroid) in self.asteroids.iter().enumerate() {
            if asteroid_hits[i] {
                continue;
            }
            survivors.push(asteroid.clone());
        }
        survivors.extend(fragments);
        self.asteroids = survivors;
    }

    fn spawn_debris(&mut self, asteroid: &Asteroid) {
        for _ in 0..DEBRIS_COUNT {
            let disk = Vec2::from_angle(gen_range(0.0, 2.0 * PI));
            let velocity = disk * DEBRIS_SPEED;
            self.debris.push(Debris::new(asteroid.position, velocity));
        }
    }

    fn reset_ship(&mut self) {
        self.ship.position = vec2(screen_width() / 2.0, screen_height() / 2.0);
        self.ship.velocity = Vec2::ZERO;
        self.ship.angle = -PI / 2.0;
        self.invuln_timer = INVULN_DURATION;
    }

    fn update_asteroids(&mut self) {
        for asteroid in &mut self.asteroids {
            asteroid.angle += asteroid.rotation_speed * self.dt;
            let target = asteroid.position + asteroid.velocity * self.dt;
            asteroid.position = wrap_position(target);
        }
    }

    fn spawn_asteroid(&mut self) {
        let width = screen_width();
        let height = screen_height();
        let side = gen_range(0, 4);
        let mut position = match side {
            0 => vec2(gen_range(0.0, width), 0.0),
            1 => vec2(width, gen_range(0.0, height)),
            2 => vec2(gen_range(0.0, width), height),
            _ => vec2(0.0, gen_range(0.0, height)),
        };

        // Avoid spawning too close to the ship
        if position.distance(self.ship.position) < SHIP_SIZE * 2.0 {
            let offset = Vec2::from_angle(gen_range(0.0, 2.0 * PI)) * (SHIP_SIZE * 3.0);
            position += offset;
        }

        let angle = gen_range(0.0, 2.0 * PI);
        let speed = gen_range(ASTEROID_MIN_SPEED, ASTEROID_MAX_SPEED);
        let velocity = Vec2::from_angle(angle) * speed;
        self.asteroids
            .push(Asteroid::new(AsteroidSize::Large, position, velocity));
    }

    fn spawn_bullet(&mut self, position: Vec2, velocity: Vec2) {
        self.bullets.push(Bullet::new(position, velocity));
    }

    fn ship_triangle(&self) -> (Vec2, Vec2, Vec2) {
        let nose = self.ship.position + Vec2::from_angle(self.ship.angle) * SHIP_SIZE;
        let rear = self.ship.position - Vec2::from_angle(self.ship.angle) * (SHIP_SIZE * 0.5);
        let perp = Vec2::from_angle(self.ship.angle + FRAC_PI_2) * (SHIP_SIZE * 0.4);
        let left = rear + perp;
        let right = rear - perp;
        (nose, left, right)
    }
}
fn wrap_position(position: Vec2) -> Vec2 {
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
fn clamp_length(value: Vec2, max: f32) -> Vec2 {
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

struct Ship {
    position: Vec2,
    velocity: Vec2,
    angle: f32,
}

impl Ship {
    fn centered() -> Self {
        Self {
            position: vec2(screen_width() / 2.0, screen_height() / 2.0),
            velocity: Vec2::ZERO,
            angle: -PI / 2.0,
        }
    }
}

#[derive(Clone)]
struct Asteroid {
    position: Vec2,
    velocity: Vec2,
    size: AsteroidSize,
    angle: f32,
    rotation_speed: f32,
    shape: Vec<Vec2>,
}

impl Asteroid {
    fn new(size: AsteroidSize, position: Vec2, velocity: Vec2) -> Self {
        Self {
            position,
            velocity,
            size,
            angle: gen_range(0.0, 2.0 * PI),
            rotation_speed: gen_range(-0.8, 0.8),
            shape: generate_shape(size),
        }
    }

    fn radius(&self) -> f32 {
        self.size.radius()
    }

    fn points(&self) -> Vec<Vec2> {
        self.shape
            .iter()
            .map(|vertex| rotate_vector(*vertex, self.angle) + self.position)
            .collect()
    }

    fn split(&self) -> Vec<Asteroid> {
        if let Some(next_size) = self.size.next() {
            let mut fragments = Vec::with_capacity(2);
            let base_len = self.velocity.length().max(ASTEROID_MIN_SPEED);
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
struct Bullet {
    position: Vec2,
    velocity: Vec2,
    ttl: f32,
}

impl Bullet {
    fn new(position: Vec2, velocity: Vec2) -> Self {
        Self {
            position,
            velocity,
            ttl: BULLET_TTL,
        }
    }
}

#[derive(Clone)]
struct Debris {
    position: Vec2,
    velocity: Vec2,
    ttl: f32,
}

impl Debris {
    fn new(position: Vec2, velocity: Vec2) -> Self {
        Self {
            position,
            velocity,
            ttl: DEBRIS_TTL,
        }
    }
}

#[derive(Clone)]
pub struct SimulationPolicy {
    pub collision_policy: super::config::CollisionPolicy,
}

impl SimulationPolicy {
    pub fn from_config(config: &GameConfig) -> Self {
        Self {
            collision_policy: config.collision_policy.clone(),
        }
    }

    pub fn degrade(&mut self) {
        self.collision_policy = super::config::CollisionPolicy::PlayerOnly;
    }
}

#[derive(Clone)]
pub struct SimulationStatus {
    pub frame: u64,
    pub last_intent: Option<ControlIntent>,
    pub asteroid_count: usize,
    pub bullet_count: usize,
    pub active_bodies: usize,
    pub primary_cooldown: f32,
    pub secondary_cooldown: f32,
    pub frame_time: f32,
    pub fps: f32,
    pub score: u32,
    pub lives: u32,
    pub game_over: bool,
}

impl Default for SimulationStatus {
    fn default() -> Self {
        Self {
            frame: 0,
            last_intent: None,
            asteroid_count: 0,
            bullet_count: 0,
            active_bodies: 1,
            primary_cooldown: 0.0,
            secondary_cooldown: 0.0,
            frame_time: 1.0 / 60.0,
            fps: 60.0,
            score: 0,
            lives: MAX_LIVES,
            game_over: false,
        }
    }
}
