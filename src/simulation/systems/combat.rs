use crate::controllers::ControlIntent;
use macroquad::prelude::Vec2;

use super::super::{
    BULLET_RADIUS, BULLET_SPEED, PRIMARY_FIRE_RATE, SECONDARY_COUNT, SECONDARY_FIRE_RATE,
    SECONDARY_SPREAD, SHIP_SIZE,
};
use super::super::model::{BulletSource, wrap_position};
use super::super::Simulation;

impl Simulation {
    pub(in crate::simulation) fn update_aliens(&mut self) {
        self.tick_alien_spawns();

        let width = macroquad::prelude::screen_width();
        let ship_pos = self.ship.position;
        let score = self.status.score;
        let mut alien_shots = Vec::new();
        for alien in &mut self.aliens {
            alien.update(self.dt, width);
            if alien.ready_to_fire() {
                let direction = alien.fire_direction(ship_pos, score);
                if direction.length_squared() > 0.0 {
                    let spawn_pos =
                        alien.position + direction * (alien.size.hit_radius() + BULLET_RADIUS + 2.0);
                    alien_shots.push((spawn_pos, direction * BULLET_SPEED));
                }
                alien.reset_fire_timer();
            }
        }

        for (position, velocity) in alien_shots {
            self.spawn_bullet(position, velocity, BulletSource::Alien);
        }
    }

    pub(in crate::simulation) fn handle_firing(&mut self, intent: ControlIntent) {
        if intent.fire_primary && self.primary_cooldown <= 0.0 {
            self.primary_cooldown = 1.0 / PRIMARY_FIRE_RATE;
            let forward = Vec2::from_angle(self.ship.angle);
            let spawn_pos = self.ship.position + forward * SHIP_SIZE;
            // Bullets inherit the ship's velocity for better feel and more interesting interactions.
            let spawn_velocity = forward * BULLET_SPEED + self.ship.velocity;
            self.spawn_bullet(spawn_pos, spawn_velocity, BulletSource::Player);
        }

        if intent.fire_secondary && self.secondary_cooldown <= 0.0 {
            self.secondary_cooldown = 1.0 / SECONDARY_FIRE_RATE;
            let base_angle = self.ship.angle;
            let center = (SECONDARY_COUNT as f32 - 1.0) * 0.5;
            for i in 0..SECONDARY_COUNT {
                let offset = (i as f32 - center) * SECONDARY_SPREAD;
                let dir = Vec2::from_angle(base_angle + offset);
                let spawn_pos = self.ship.position + dir * SHIP_SIZE;
                self.spawn_bullet(spawn_pos, dir * BULLET_SPEED, BulletSource::Player);
            }
        }
    }

    pub(in crate::simulation) fn update_bullets(&mut self) {
        self.bullets.retain_mut(|bullet| {
            bullet.ttl -= self.dt;
            if bullet.ttl <= 0.0 {
                return false;
            }
            bullet.position = wrap_position(bullet.position + bullet.velocity * self.dt);
            true
        });
    }

    pub(in crate::simulation) fn update_debris(&mut self) {
        self.debris.retain_mut(|debris| {
            debris.ttl -= self.dt;
            if debris.ttl <= 0.0 {
                return false;
            }
            debris.position = wrap_position(debris.position + debris.velocity * self.dt);
            true
        });
    }
}
