use crate::ai::WorldSnapshot;
use crate::controllers::{ControlIntent, Controller};
use macroquad::prelude::{KeyCode, is_key_down};

pub struct HumanController;

impl Default for HumanController {
    fn default() -> Self {
        HumanController
    }
}

impl Controller for HumanController {
    fn tick(&mut self, _world: &WorldSnapshot, _dt: f32) -> ControlIntent {
        let mut turn = 0.0;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            turn -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            turn += 1.0;
        }

        let thrust = if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            1.0
        } else {
            0.0
        };

        ControlIntent {
            thrust,
            turn,
            fire_primary: is_key_down(KeyCode::Space),
            fire_secondary: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
        }
    }
}
