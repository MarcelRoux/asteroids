pub mod human;

use crate::ai::WorldSnapshot;

#[derive(Clone, Copy, Default)]
pub struct ControlIntent {
    pub thrust: f32,
    pub turn: f32,
    pub fire_primary: bool,
    pub fire_secondary: bool,
}

pub trait Controller {
    fn tick(&mut self, world: &WorldSnapshot, dt: f32) -> ControlIntent;
}
