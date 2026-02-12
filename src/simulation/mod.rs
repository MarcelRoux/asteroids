use crate::ai::{AsteroidSnapshot, WorldSnapshot};
use crate::config::{GameConfig, PlayerControllerMode};
use crate::controllers::{ControlIntent, Controller};
use crate::stats::RunStats;
use macroquad::prelude::{Color, Vec2};
use std::f32::consts::PI;

const SHIP_THRUST: f32 = 400.0;
const SHIP_MAX_SPEED: f32 = 320.0;
const SHIP_DRAG: f32 = 0.25;
const SHIP_ROTATION_SPEED: f32 = 3.0;
const SHIP_SIZE: f32 = 14.0;
const SHIP_STROKE: f32 = 2.0;
const SAUCER_STROKE: f32 = 2.0;
const SMALL_ALIEN_DRAW_SCALE: f32 = 12.0;
const LARGE_ALIEN_DRAW_SCALE: f32 = 24.0;
const INVULNERABILITY_DURATION: f32 = 3.0;
const SHIP_DRAW_OFFSET: f32 = -std::f32::consts::PI / 2.0;

// TODO: Increased 10x for the time being due to much more powerful weapons.
const EXTRA_LIFE_SCORE_STEP: u32 = 10_000;

const ALIEN_SPAWN_SCORE_THRESHOLD: u32 = 40_000;
const ALIEN_SPAWN_INTERVAL: f32 = 5.0;
const SMALL_ALIEN_Y: f32 = 110.0;
const LARGE_ALIEN_Y: f32 = 70.0;
const SMALL_ALIEN_SPEED: f32 = 160.0;
const LARGE_ALIEN_SPEED: f32 = 96.0;
const SMALL_ALIEN_FIRE_INTERVAL: f32 = 1.1;
const LARGE_ALIEN_FIRE_INTERVAL: f32 = 1.9;
const MAX_SMALL_ALIENS: usize = 2;
const MAX_LARGE_ALIENS: usize = 1;
const SMALL_ALIEN_SCORE: u32 = 1000;
const LARGE_ALIEN_SCORE: u32 = 200;

const ASTEROID_MIN_SPEED: f32 = 20.0;
const ASTEROID_MAX_SPEED: f32 = 90.0;
const ASTEROID_SPAWN_INTERVAL: f32 = 2.5;
const BULLET_SPEED: f32 = 520.0;
const BULLET_RADIUS: f32 = 2.0;
const BULLET_TTL: f32 = 2.0;
const PRIMARY_FIRE_RATE: f32 = 10.0;
const SECONDARY_COUNT: usize = 21;
const SECONDARY_FIRE_RATE: f32 = PRIMARY_FIRE_RATE / SECONDARY_COUNT as f32;
const SECONDARY_SPREAD: f32 = PI / 36.0;
const MAX_LIVES: u32 = 3;
const ASTEROID_SCORE_BASE: u32 = 100;
const DEBRIS_TTL: f32 = 1.0;
const DEBRIS_SPEED: f32 = 120.0;
const DEBRIS_COUNT: usize = 6;
const DEBRIS_COLOR: Color = Color::new(1.0, 0.75, 0.3, 1.0);
const ALIEN_DEBRIS_COLOR: Color = Color::new(1.0, 0.2, 0.3, 1.0);
const PLAYER_DEBRIS_COLOR: Color = Color::new(0.35, 0.8, 1.0, 1.0);

const TARGET_FPS: f32 = 60.0;

mod model;
use self::model::*;
mod render;
mod systems;

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
    invulnerability_timer: f32,
    invulnerability_enabled: bool,
    run_stats: RunStats,
    aliens: Vec<Alien>,
    alien_spawn_acc: f32,
    next_extra_life_score: u32,
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
            dt: 1.0 / TARGET_FPS,
            status: SimulationStatus::default(),
            invulnerability_timer: INVULNERABILITY_DURATION,
            invulnerability_enabled: false,
            run_stats: RunStats::default(),
            aliens: Vec::new(),
            alien_spawn_acc: 0.0,
            next_extra_life_score: EXTRA_LIFE_SCORE_STEP,
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

    pub fn toggle_invulnerability(&mut self) {
        self.invulnerability_enabled = !self.invulnerability_enabled;
        self.status.invulnerability_enabled = self.invulnerability_enabled;
    }

    fn record_player_shot(&mut self) {
        self.run_stats.shots_fired = self.run_stats.shots_fired.saturating_add(1);
    }

    fn record_player_hit(&mut self, target: HitTarget) {
        self.run_stats.shots_hit = self.run_stats.shots_hit.saturating_add(1);
        match target {
            HitTarget::LargeAsteroid => {
                self.run_stats.hits_large_asteroid =
                    self.run_stats.hits_large_asteroid.saturating_add(1)
            }
            HitTarget::MediumAsteroid => {
                self.run_stats.hits_medium_asteroid =
                    self.run_stats.hits_medium_asteroid.saturating_add(1)
            }
            HitTarget::SmallAsteroid => {
                self.run_stats.hits_small_asteroid =
                    self.run_stats.hits_small_asteroid.saturating_add(1)
            }
            HitTarget::LargeAlien => {
                self.run_stats.hits_large_alien = self.run_stats.hits_large_alien.saturating_add(1)
            }
            HitTarget::SmallAlien => {
                self.run_stats.hits_small_alien = self.run_stats.hits_small_alien.saturating_add(1)
            }
        }
    }

    pub fn step(&mut self) {
        self.status.frame += 1;
        let intent = self.status.last_intent.unwrap_or_default();
        self.update_ship(intent);
        self.handle_firing(intent);
        self.update_asteroids();
        self.tick_asteroid_spawns();

        self.update_bullets();
        self.update_debris();
        self.update_aliens();
        self.resolve_collisions();

        self.primary_cooldown = (self.primary_cooldown - self.dt).max(0.0);
        self.secondary_cooldown = (self.secondary_cooldown - self.dt).max(0.0);
        self.invulnerability_timer = (self.invulnerability_timer - self.dt).max(0.0);

        self.status.asteroid_count = self.asteroids.len();
        self.status.bullet_count = self.bullets.len();
        self.status.active_bodies = 1 + self.asteroids.len() + self.bullets.len();
        self.status.primary_cooldown = self.primary_cooldown;
        self.status.secondary_cooldown = self.secondary_cooldown;
        self.status.frame_time = self.dt;
        self.status.fps = 1.0 / self.dt;
        self.status.lives = self.lives;
        self.status.game_over = self.lives == 0;
        self.status.invulnerability_enabled = self.invulnerability_enabled;
        self.status.run_stats = self.run_stats.clone();
    }

    pub fn policy(&mut self) -> &mut SimulationPolicy {
        &mut self.policy
    }

    pub fn status(&self) -> SimulationStatus {
        self.status.clone()
    }

    fn spawn_bullet(&mut self, position: Vec2, velocity: Vec2, source: BulletSource) {
        self.bullets.push(Bullet::new(position, velocity, source));
        if source == BulletSource::Player {
            self.record_player_shot();
        }
    }

    // Rendering helpers live in `render.rs`.
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
    pub invulnerability_enabled: bool,
    pub run_stats: RunStats,
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
            frame_time: 1.0 / TARGET_FPS,
            fps: TARGET_FPS,
            score: 0,
            lives: MAX_LIVES,
            game_over: false,
            invulnerability_enabled: false,
            run_stats: RunStats::default(),
        }
    }
}
