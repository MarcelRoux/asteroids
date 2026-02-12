use macroquad::prelude::{Color, Vec2, screen_height, screen_width, vec2};
use macroquad::rand::gen_range;
use std::f32::consts::PI;

use super::super::{
    ALIEN_DEBRIS_COLOR, BULLET_RADIUS, DEBRIS_COLOR, DEBRIS_COUNT, DEBRIS_SPEED,
    EXTRA_LIFE_SCORE_STEP, INVULNERABILITY_DURATION, PLAYER_DEBRIS_COLOR, SHIP_SIZE,
};
use super::super::model::{AsteroidSize, BulletSource, Debris, HitTarget};
use super::super::Simulation;

impl Simulation {
    pub(in crate::simulation) fn award_extra_lives(&mut self) {
        while self.status.score >= self.next_extra_life_score {
            self.lives = self.lives.saturating_add(1);
            self.next_extra_life_score = self
                .next_extra_life_score
                .saturating_add(EXTRA_LIFE_SCORE_STEP);
        }
    }

    pub(in crate::simulation) fn resolve_collisions(&mut self) {
        let mut bullet_hits = vec![false; self.bullets.len()];
        let mut asteroid_hits = vec![false; self.asteroids.len()];
        let mut alien_hits = vec![false; self.aliens.len()];
        let mut pending_hits = Vec::new();
        let ship_radius = SHIP_SIZE * 0.9;
        let mut fragments = Vec::new();
        let mut earned_score: u32 = 0;
        let mut destroyed_asteroids = Vec::new();
        let mut destroyed_aliens = Vec::new();

        for (bi, bullet) in self.bullets.iter().enumerate() {
            if bullet_hits[bi] {
                continue;
            }

            let mut handled = false;
            for (ai, asteroid) in self.asteroids.iter().enumerate() {
                if asteroid_hits[ai] {
                    continue;
                }
                let radius_sum = asteroid.radius() + BULLET_RADIUS;
                if bullet.position.distance_squared(asteroid.position) <= radius_sum * radius_sum {
                    bullet_hits[bi] = true;
                    asteroid_hits[ai] = true;
                    fragments.extend(asteroid.split());
                    destroyed_asteroids.push(asteroid.clone());
                    if bullet.source == BulletSource::Player {
                        earned_score = earned_score.saturating_add(asteroid.size.score());
                        let target = match asteroid.size {
                            AsteroidSize::Large => HitTarget::LargeAsteroid,
                            AsteroidSize::Medium => HitTarget::MediumAsteroid,
                            AsteroidSize::Small => HitTarget::SmallAsteroid,
                        };
                        pending_hits.push(target);
                    }
                    handled = true;
                    break;
                }
            }

            if handled {
                continue;
            }

            for (ai, alien) in self.aliens.iter().enumerate() {
                if alien_hits[ai] {
                    continue;
                }
                let radius_sum = alien.size.hit_radius() + BULLET_RADIUS;
                if bullet.position.distance_squared(alien.position) <= radius_sum * radius_sum {
                    bullet_hits[bi] = true;
                    alien_hits[ai] = true;
                    earned_score = earned_score.saturating_add(alien.size.score_value());
                    destroyed_aliens.push(alien.position);
                    if bullet.source == BulletSource::Player {
                        pending_hits.push(alien.size.hit_target());
                    }
                    handled = true;
                    break;
                }
            }

            if handled {
                continue;
            }
        }

        let mut ship_hit = false;
        for (ai, asteroid) in self.asteroids.iter().enumerate() {
            let radius_sum = asteroid.radius() + ship_radius;
            if self.invulnerability_timer <= 0.0
                && self.ship.position.distance_squared(asteroid.position) <= radius_sum * radius_sum
            {
                asteroid_hits[ai] = true;
                if !self.invulnerability_enabled {
                    ship_hit = true;
                }
            }
        }

        for (bi, bullet) in self.bullets.iter().enumerate() {
            if bullet_hits[bi] {
                continue;
            }
            if bullet.source == BulletSource::Alien
                && self.invulnerability_timer <= 0.0
                && !self.invulnerability_enabled
                && self.ship.position.distance_squared(bullet.position) <= ship_radius * ship_radius
            {
                bullet_hits[bi] = true;
                ship_hit = true;
            }
        }

        if ship_hit {
            if self.lives > 0 {
                self.lives -= 1;
            }
            self.spawn_debris(self.ship.position, PLAYER_DEBRIS_COLOR);
            if self.lives > 0 {
                self.reset_ship();
            }
        }

        self.status.score = self.status.score.saturating_add(earned_score);
        self.status.invulnerability_enabled = self.invulnerability_enabled;
        self.award_extra_lives();

        for asteroid in destroyed_asteroids {
            self.spawn_debris(asteroid.position, DEBRIS_COLOR);
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

        let mut remaining_aliens = Vec::new();
        for (i, alien) in self.aliens.iter().enumerate() {
            if alien_hits[i] {
                continue;
            }
            remaining_aliens.push(alien.clone());
        }
        self.aliens = remaining_aliens;
        for pos in destroyed_aliens {
            self.spawn_debris(pos, ALIEN_DEBRIS_COLOR);
        }

        for hit in pending_hits {
            self.record_player_hit(hit);
        }

        // Remove bullets that collided.
        let mut remaining_bullets = Vec::new();
        for (i, bullet) in self.bullets.iter().enumerate() {
            if bullet_hits[i] {
                continue;
            }
            remaining_bullets.push(bullet.clone());
        }
        self.bullets = remaining_bullets;
    }

    pub(in crate::simulation) fn spawn_debris(&mut self, origin: Vec2, color: Color) {
        for _ in 0..DEBRIS_COUNT {
            let disk = Vec2::from_angle(gen_range(0.0, 2.0 * PI));
            let velocity = disk * DEBRIS_SPEED;
            self.debris.push(Debris::new(origin, velocity, color));
        }
    }

    pub(in crate::simulation) fn reset_ship(&mut self) {
        self.ship.position = vec2(screen_width() / 2.0, screen_height() / 2.0);
        self.ship.velocity = Vec2::ZERO;
        self.ship.angle = -PI / 2.0;
        self.invulnerability_timer = INVULNERABILITY_DURATION;
    }
}
