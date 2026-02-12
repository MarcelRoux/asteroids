use crate::controllers::ControlIntent;
use macroquad::prelude::Vec2;

use super::super::{SHIP_DRAG, SHIP_MAX_SPEED, SHIP_ROTATION_SPEED, SHIP_THRUST};
use super::super::model::{clamp_length, wrap_position};
use super::super::Simulation;

impl Simulation {
    pub(in crate::simulation) fn update_ship(&mut self, intent: ControlIntent) {
        self.ship.angle += intent.turn * SHIP_ROTATION_SPEED * self.dt;
        let forward = Vec2::from_angle(self.ship.angle);

        if intent.thrust > 0.0 {
            self.ship.velocity += forward * (intent.thrust * SHIP_THRUST * self.dt);
        }

        self.ship.velocity -= self.ship.velocity * SHIP_DRAG * self.dt;
        self.ship.velocity = clamp_length(self.ship.velocity, SHIP_MAX_SPEED);
        self.ship.position = wrap_position(self.ship.position + self.ship.velocity * self.dt);
    }

    pub(in crate::simulation) fn update_asteroids(&mut self) {
        for asteroid in &mut self.asteroids {
            asteroid.angle += asteroid.rotation_speed * self.dt;
            let target = asteroid.position + asteroid.velocity * self.dt;
            asteroid.position = wrap_position(target);
        }
    }
}
