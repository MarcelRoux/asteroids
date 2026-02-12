mod session;
mod state;

pub use state::AppState;

use crate::config::{self, AiProfile, GameConfig};
use crate::ui::menu;
use macroquad::prelude::*;

use self::session::Session;

const PRESET_LABELS: [&str; 3] = ["Classic", "Arcade Upgrades", "AI Autopilot"];
const GAME_OVER_NAME_MAX: usize = 12;

pub struct App {
    config: GameConfig,
    presets: Vec<GameConfig>,
    preset_index: usize,
    preset_label: String,
    session: Session,
    state: AppState,
    stats_visible: bool,
    autopilot_engaged: bool,
    autopilot_profile: AiProfile,
    game_over_name: String,
    name_cursor: usize,
}

impl App {
    pub fn new() -> Self {
        let config = GameConfig::default();
        let presets = config::presets::default_presets();
        debug_assert_eq!(presets.len(), PRESET_LABELS.len());
        let mut session = Session::new(&config);
        session.set_controller(false, AiProfile::Balanced);
        Self {
            config,
            presets,
            preset_index: 0,
            preset_label: PRESET_LABELS[0].to_string(),
            session,
            state: AppState::MainMenu,
            stats_visible: true,
            autopilot_engaged: false,
            autopilot_profile: AiProfile::Balanced,
            game_over_name: String::new(),
            name_cursor: 0,
        }
    }

    pub async fn tick(&mut self) -> bool {
        clear_background(BLACK);
        let mut continue_running = true;

        match self.state {
            AppState::Playing => {
                let snapshot = self.session.simulation().snapshot();
                let dt = self.session.simulation().dt();
                let intent = self.session.controller().tick(&snapshot, dt);

                {
                    let simulation = self.session.simulation_mut();
                    simulation.apply_intent(intent);
                    simulation.step();
                }

                let status = self.session.simulation().status();
                let fps = status.fps;
                let should_degrade = {
                    let guard = self.session.performance_guard();
                    guard.observe(fps);
                    guard.should_degrade()
                };

                if should_degrade {
                    self.session.simulation_mut().policy().degrade();
                }

                self.session.simulation().draw_debug();
                menu::draw_score_display(&status);
                if self.stats_visible {
                    menu::draw_stats_overlay(&self.config, &status);
                }
                menu::draw_autopilot_status(self.autopilot_engaged, profile_label(&self.autopilot_profile));

                if status.game_over {
                    self.game_over_name = default_game_over_name(
                        self.autopilot_engaged,
                        self.autopilot_profile,
                    );
                    self.name_cursor = self.game_over_name.len();
                    drain_char_input();
                    self.state = AppState::GameOver;
                } else {
                    if is_key_pressed(KeyCode::Escape) {
                        self.finish_run();
                        self.state = AppState::MainMenu;
                    }
                    if is_key_pressed(KeyCode::P) {
                        if self.autopilot_engaged {
                            self.autopilot_profile = cycle_profile(self.autopilot_profile);
                            self.session.set_controller(true, self.autopilot_profile);
                        } else {
                            self.state = AppState::Paused;
                        }
                    }
                    if is_key_pressed(KeyCode::I) {
                        self.session.simulation_mut().toggle_invulnerability();
                    }
                    if is_key_pressed(KeyCode::T) {
                        self.stats_visible = !self.stats_visible;
                    }
                    if is_key_pressed(KeyCode::U) {
                        self.autopilot_engaged = !self.autopilot_engaged;
                        self.session.set_controller(self.autopilot_engaged, self.autopilot_profile);
                    }
                }
            }
            AppState::MainMenu => {
                menu::draw_main_menu();
                if is_key_pressed(KeyCode::P) {
                    self.session.reset(&self.config, self.autopilot_engaged, self.autopilot_profile);
                    self.state = AppState::Playing;
                }
                if is_key_pressed(KeyCode::O) {
                    self.state = AppState::Options;
                }
                if is_key_pressed(KeyCode::L) {
                    self.state = AppState::Leaderboard;
                }
                if is_key_pressed(KeyCode::Escape) {
                    continue_running = false;
                }
            }
            AppState::Options => {
                menu::draw_options_menu(&self.config, &self.preset_label);
                if is_key_pressed(KeyCode::C) {
                    self.config.cycle_collision_policy();
                    self.preset_label = "Custom".to_string();
                }
                if is_key_pressed(KeyCode::K) {
                    self.config.cycle_physics_mode();
                    self.preset_label = "Custom".to_string();
                }
                if is_key_pressed(KeyCode::F) {
                    self.config.cycle_fragmentation_mode();
                    self.preset_label = "Custom".to_string();
                }
                if is_key_pressed(KeyCode::L) {
                    self.config.cycle_leaderboard_mode();
                    self.preset_label = "Custom".to_string();
                }
                if is_key_pressed(KeyCode::G) {
                    self.config.toggle_upgrades();
                    self.preset_label = "Custom".to_string();
                }
                if is_key_pressed(KeyCode::Y) {
                    self.preset_index = (self.preset_index + 1) % self.presets.len();
                    self.config = self.presets[self.preset_index].clone();
                    self.preset_label = PRESET_LABELS[self.preset_index].to_string();
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
                    self.state = AppState::MainMenu;
                }
            }
            AppState::Leaderboard => {
                menu::draw_leaderboard_menu(self.session.leaderboard());
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
                    self.state = AppState::MainMenu;
                }
            }
            AppState::Paused => {
                self.session.simulation().draw_debug();
                let status = self.session.simulation().status();
                menu::draw_score_display(&status);
                if self.stats_visible {
                    menu::draw_stats_overlay(&self.config, &status);
                }
                menu::draw_autopilot_status(self.autopilot_engaged, profile_label(&self.autopilot_profile));
                if status.game_over {
                    self.game_over_name = default_game_over_name(
                        self.autopilot_engaged,
                        self.autopilot_profile,
                    );
                    self.name_cursor = self.game_over_name.len();
                    drain_char_input();
                    self.state = AppState::GameOver;
                } else {
                    draw_text(
                        "PAUSED — press P to resume, Esc to end run",
                        screen_width() / 2.0 - 220.0,
                        screen_height() / 2.0,
                        30.0,
                        WHITE,
                    );
                    draw_text(
                        "(P also cycles AI profile when autopilot is enabled)",
                        screen_width() / 2.0 - 260.0,
                        screen_height() / 2.0 + 34.0,
                        20.0,
                        WHITE,
                    );

                    if is_key_pressed(KeyCode::P) {
                        if self.autopilot_engaged {
                            self.autopilot_profile = cycle_profile(self.autopilot_profile);
                            self.session.set_controller(true, self.autopilot_profile);
                        } else {
                            self.state = AppState::Playing;
                        }
                    }
                    if is_key_pressed(KeyCode::I) {
                        self.session.simulation_mut().toggle_invulnerability();
                    }
                    if is_key_pressed(KeyCode::Escape) {
                        self.finish_run();
                        self.state = AppState::MainMenu;
                    }
                    if is_key_pressed(KeyCode::T) {
                        self.stats_visible = !self.stats_visible;
                    }
                    if is_key_pressed(KeyCode::U) {
                        self.autopilot_engaged = !self.autopilot_engaged;
                        self.session.set_controller(self.autopilot_engaged, self.autopilot_profile);
                    }
                }
            }
            AppState::GameOver => {
                self.session.simulation().draw_debug();
                let status = self.session.simulation().status();
                menu::draw_score_display(&status);
                if self.stats_visible {
                    menu::draw_stats_overlay(&self.config, &status);
                }
                menu::draw_autopilot_status(self.autopilot_engaged, profile_label(&self.autopilot_profile));
                menu::draw_game_over(
                    status.score,
                    &self.game_over_name,
                    self.name_cursor,
                    GAME_OVER_NAME_MAX,
                );

                while let Some(c) = get_char_pressed() {
                    if (c.is_ascii_alphanumeric() || c == ' ')
                        && self.game_over_name.len() < GAME_OVER_NAME_MAX
                    {
                        self.game_over_name.push(c.to_ascii_uppercase());
                        self.name_cursor = self.game_over_name.len();
                    }
                }
                if is_key_pressed(KeyCode::Backspace) {
                    if self.game_over_name.pop().is_some() {
                        self.name_cursor = self.game_over_name.len();
                    }
                }

                if is_key_pressed(KeyCode::Enter) {
                    if status.score > 0 {
                        let name = sanitize_run_name(
                            &self.game_over_name,
                            self.autopilot_engaged,
                            self.autopilot_profile,
                        );
                        self.session
                            .leaderboard_mut()
                            .submit(&name, status.score, status.run_stats.clone());
                        self.session.leaderboard_mut().save();
                    }
                    self.finish_run();
                    self.state = AppState::MainMenu;
                }
                if is_key_pressed(KeyCode::Escape) {
                    self.finish_run();
                    self.state = AppState::MainMenu;
                }
                if is_key_pressed(KeyCode::U) {
                    self.autopilot_engaged = !self.autopilot_engaged;
                    self.session.set_controller(self.autopilot_engaged, self.autopilot_profile);
                }
                if is_key_pressed(KeyCode::P) && self.autopilot_engaged {
                    self.autopilot_profile = cycle_profile(self.autopilot_profile);
                    self.session.set_controller(true, self.autopilot_profile);
                }
            }
        }

        next_frame().await;
        continue_running
    }

    fn finish_run(&mut self) {
        self.session
            .reset(&self.config, self.autopilot_engaged, self.autopilot_profile);
    }
}

fn cycle_profile(current: AiProfile) -> AiProfile {
    match current {
        AiProfile::Casual => AiProfile::Balanced,
        AiProfile::Balanced => AiProfile::Veteran,
        AiProfile::Veteran => AiProfile::Casual,
    }
}

fn profile_label(profile: &AiProfile) -> &'static str {
    match profile {
        AiProfile::Casual => "Casual",
        AiProfile::Balanced => "Balanced",
        AiProfile::Veteran => "Veteran",
    }
}

fn sanitize_run_name(input: &str, autopilot: bool, profile: AiProfile) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return default_game_over_name(autopilot, profile);
    }

    trimmed
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ')
        .take(GAME_OVER_NAME_MAX)
        .collect()
}

fn default_game_over_name(autopilot: bool, profile: AiProfile) -> String {
    if autopilot {
        format!("AI {}", profile_label(&profile))
    } else {
        "PLAYER".to_string()
    }
}

fn drain_char_input() {
    // Macroquad queues typed characters until polled via `get_char_pressed()`.
    // Drain it on state transitions so gameplay keystrokes don't appear in the GameOver name entry.
    while get_char_pressed().is_some() {}
}
