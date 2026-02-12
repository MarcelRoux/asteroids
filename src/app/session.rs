use crate::ai::AiController;
use crate::config::{AiProfile, GameConfig};
use crate::controllers::Controller;
use crate::controllers::human::HumanController;
use crate::eval::PerformanceGuard;
use crate::scoreboard::Leaderboard;
use crate::simulation::Simulation;

pub struct Session {
    simulation: Simulation,
    performance_guard: PerformanceGuard,
    leaderboard: Leaderboard,
}

impl Session {
    pub fn new(config: &GameConfig) -> Self {
        Self {
            simulation: Simulation::new(config.clone()),
            performance_guard: PerformanceGuard::new(),
            leaderboard: Leaderboard::load(),
        }
    }

    pub fn reset(&mut self, config: &GameConfig, autopilot: bool, profile: AiProfile) {
        self.simulation = Simulation::new(config.clone());
        self.performance_guard = PerformanceGuard::new();
        self.set_controller(autopilot, profile);
    }

    pub fn set_controller(&mut self, autopilot: bool, profile: AiProfile) {
        if autopilot {
            self.simulation.set_controller(Box::new(AiController::new(profile)));
        } else {
            self.simulation.set_controller(Box::new(HumanController::default()));
        }
    }

    pub fn simulation(&self) -> &Simulation {
        &self.simulation
    }

    pub fn simulation_mut(&mut self) -> &mut Simulation {
        &mut self.simulation
    }

    pub fn controller(&mut self) -> &mut dyn Controller {
        self.simulation.controller()
    }

    pub fn performance_guard(&mut self) -> &mut PerformanceGuard {
        &mut self.performance_guard
    }

    pub fn leaderboard(&self) -> &Leaderboard {
        &self.leaderboard
    }

    pub fn leaderboard_mut(&mut self) -> &mut Leaderboard {
        &mut self.leaderboard
    }
}
