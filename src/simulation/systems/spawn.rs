use macroquad::prelude::{Vec2, screen_height, screen_width, vec2};
use macroquad::rand::gen_range;
use std::f32::consts::PI;

use super::super::{
    ALIEN_SPAWN_INTERVAL, ALIEN_SPAWN_SCORE_THRESHOLD, ASTEROID_MAX_SPEED, ASTEROID_MIN_SPEED,
    ASTEROID_SPAWN_INTERVAL, MAX_LARGE_ALIENS, MAX_SMALL_ALIENS, SHIP_SIZE,
};
use super::super::model::{Alien, AlienSize, Asteroid, AsteroidSize};
use super::super::Simulation;

impl Simulation {
    pub(in crate::simulation) fn tick_asteroid_spawns(&mut self) {
        self.spawn_acc += self.dt;
        while self.spawn_acc >= ASTEROID_SPAWN_INTERVAL {
            self.spawn_acc -= ASTEROID_SPAWN_INTERVAL;
            self.spawn_asteroid();
        }
    }

    pub(in crate::simulation) fn spawn_asteroid(&mut self) {
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

    pub(in crate::simulation) fn tick_alien_spawns(&mut self) {
        let width = screen_width();
        self.alien_spawn_acc += self.dt;
        while self.alien_spawn_acc >= ALIEN_SPAWN_INTERVAL {
            self.alien_spawn_acc -= ALIEN_SPAWN_INTERVAL;
            self.maybe_spawn_alien(width);
        }
    }

    pub(in crate::simulation) fn maybe_spawn_alien(&mut self, width: f32) {
        if self.status.score < ALIEN_SPAWN_SCORE_THRESHOLD {
            return;
        }

        let small_count = self
            .aliens
            .iter()
            .filter(|alien| matches!(alien.size, AlienSize::Small))
            .count();
        let large_count = self
            .aliens
            .iter()
            .filter(|alien| matches!(alien.size, AlienSize::Large))
            .count();

        if small_count >= MAX_SMALL_ALIENS && large_count >= MAX_LARGE_ALIENS {
            return;
        }

        let spawn_small = (gen_range(0.0, 1.0) < 0.65 && small_count < MAX_SMALL_ALIENS)
            || large_count >= MAX_LARGE_ALIENS;
        let size = if spawn_small {
            AlienSize::Small
        } else if large_count < MAX_LARGE_ALIENS {
            AlienSize::Large
        } else {
            return;
        };

        let direction = if gen_range(0.0, 1.0) < 0.5 { 1.0 } else { -1.0 };
        let start_x = if direction > 0.0 { 30.0 } else { width - 30.0 };
        self.aliens
            .push(Alien::new(size, direction, start_x, width));
    }
}
