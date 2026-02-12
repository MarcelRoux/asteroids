use crate::render::shapes::{
    draw_shape, saucer_large_lines, saucer_small_lines, ship_lines,
};
use macroquad::prelude::{Color, Vec2, WHITE, draw_circle, draw_line};
use macroquad::rand::gen_range;

use super::{
    BULLET_RADIUS, INVULNERABILITY_DURATION, LARGE_ALIEN_DRAW_SCALE, SAUCER_STROKE, SHIP_DRAW_OFFSET,
    SHIP_SIZE, SHIP_STROKE, SMALL_ALIEN_DRAW_SCALE,
};
use super::model::{AlienSize};
use super::Simulation;

impl Simulation {
    pub fn draw_debug(&self) {
        for asteroid in &self.asteroids {
            let points = asteroid.points();
            if points.len() > 1 {
                for i in 0..points.len() {
                    let a = points[i];
                    let b = points[(i + 1) % points.len()];
                    draw_line(a.x, a.y, b.x, b.y, 2.0, macroquad::prelude::LIGHTGRAY);
                }
            }
        }

        let ship_segments = ship_lines(SHIP_SIZE);
        draw_shape(
            &ship_segments,
            self.ship.position,
            self.ship.angle + SHIP_DRAW_OFFSET,
            SHIP_STROKE,
            WHITE,
            true,
        );
        self.draw_thruster();

        if self.invulnerability_timer > 0.0 || self.invulnerability_enabled {
            let alpha = if self.invulnerability_timer > 0.0 {
                ((self.invulnerability_timer / INVULNERABILITY_DURATION) * 0.8).clamp(0.2, 0.8)
            } else {
                0.6
            };
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
            draw_circle(debris.position.x, debris.position.y, 2.0, debris.color);
        }

        for alien in &self.aliens {
            let lines = match alien.size {
                AlienSize::Small => saucer_small_lines(SMALL_ALIEN_DRAW_SCALE),
                AlienSize::Large => saucer_large_lines(LARGE_ALIEN_DRAW_SCALE),
            };
            draw_shape(&lines, alien.position, alien.angle, SAUCER_STROKE, WHITE, true);
        }
    }

    fn draw_thruster(&self) {
        let intent = self.status.last_intent.unwrap_or_default();
        if intent.thrust <= 0.0 {
            return;
        }
        let left = self.world_ship_point(-0.5, 0.15);
        let right = self.world_ship_point(-0.5, -0.15);
        let jitter = gen_range(-0.05, 0.05);
        let tip = self.world_ship_point(-0.95 - intent.thrust * 0.2 + jitter, 0.0);
        let flame_alpha = 0.7 * intent.thrust;
        draw_line(
            left.x,
            left.y,
            tip.x,
            tip.y,
            2.0,
            Color::new(1.0, 1.0, 1.0, flame_alpha),
        );
        draw_line(
            right.x,
            right.y,
            tip.x,
            tip.y,
            2.0,
            Color::new(1.0, 1.0, 1.0, flame_alpha),
        );
        draw_line(
            left.x,
            left.y,
            right.x,
            right.y,
            1.5,
            Color::new(1.0, 1.0, 1.0, flame_alpha * 0.8),
        );
    }

    fn world_ship_point(&self, x: f32, y: f32) -> Vec2 {
        let scale = SHIP_SIZE;
        let cos = self.ship.angle.cos();
        let sin = self.ship.angle.sin();
        let local = Vec2::new(x * scale, y * scale);
        Vec2::new(local.x * cos - local.y * sin, local.x * sin + local.y * cos) + self.ship.position
    }
}

