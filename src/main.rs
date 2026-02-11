use macroquad::prelude::*;

mod ai;
mod config;
mod controllers;
mod eval;
mod scoreboard;
mod simulation;
mod ui;

use config::{AiProfile, GameConfig};
use eval::PerformanceGuard;
use scoreboard::Leaderboard;
use simulation::Simulation;
use ui::menu;

const PRESET_LABELS: [&str; 3] = ["Classic", "Arcade Upgrades", "AI Autopilot"];

#[derive(PartialEq, Eq)]
enum AppState {
    MainMenu,
    Options,
    Leaderboard,
    Playing,
    Paused,
    GameOver,
}

#[macroquad::main("Asteroids Systems")]
async fn main() {
    let mut config = GameConfig::default();
    let presets = crate::config::presets::default_presets();
    debug_assert_eq!(presets.len(), PRESET_LABELS.len());
    let mut preset_index = 0;
    let mut preset_label = PRESET_LABELS[preset_index].to_string();
    let mut simulation = Simulation::new(config.clone());
    let mut performance_guard = PerformanceGuard::new();
    let mut leaderboard = Leaderboard::load();
    let mut state = AppState::MainMenu;
    let mut stats_visible = true;
    let mut autopilot_engaged = false;
    let mut autopilot_profile = AiProfile::Balanced;

    loop {
        clear_background(BLACK);
        match state {
            AppState::Playing => {
                let snapshot = simulation.snapshot();
                let dt = simulation.dt();
                let intent = simulation.controller().tick(&snapshot, dt);

                simulation.apply_intent(intent);
                simulation.step();
                performance_guard.observe(&simulation);

                if performance_guard.should_degrade() {
                    simulation.policy().degrade();
                }

                simulation.draw_debug();
                let status = simulation.status();
                menu::draw_score_display(&status);
                if stats_visible {
                    menu::draw_stats_overlay(&config, &status);
                }
                menu::draw_autopilot_status(autopilot_engaged, profile_label(&autopilot_profile));

                if status.game_over {
                    state = AppState::GameOver;
                    continue;
                }

                if is_key_pressed(KeyCode::Escape) {
                    finish_run(
                        &mut simulation,
                        &mut performance_guard,
                        &config,
                        &mut leaderboard,
                        autopilot_engaged,
                        autopilot_profile,
                    );
                    state = AppState::MainMenu;
                }
                if is_key_pressed(KeyCode::P) {
                    state = AppState::Paused;
                }
                if is_key_pressed(KeyCode::T) {
                    stats_visible = !stats_visible;
                }
                if is_key_pressed(KeyCode::U) {
                    autopilot_engaged = !autopilot_engaged;
                    set_controller_for_mode(&mut simulation, autopilot_engaged, autopilot_profile);
                }
                if is_key_pressed(KeyCode::I) && autopilot_engaged {
                    autopilot_profile = cycle_profile(autopilot_profile);
                    set_controller_for_mode(&mut simulation, true, autopilot_profile);
                }
            }
            AppState::MainMenu => {
                menu::draw_main_menu();
                if is_key_pressed(KeyCode::P) {
                    simulation = Simulation::new(config.clone());
                    performance_guard = PerformanceGuard::new();
                    set_controller_for_mode(&mut simulation, autopilot_engaged, autopilot_profile);
                    state = AppState::Playing;
                }
                if is_key_pressed(KeyCode::O) {
                    state = AppState::Options;
                }
                if is_key_pressed(KeyCode::L) {
                    state = AppState::Leaderboard;
                }
                if is_key_pressed(KeyCode::Escape) {
                    break;
                }
            }
            AppState::Options => {
                menu::draw_options_menu(&config, &preset_label);
                if is_key_pressed(KeyCode::C) {
                    config.cycle_collision_policy();
                    preset_label = "Custom".to_string();
                }
                if is_key_pressed(KeyCode::K) {
                    config.cycle_physics_mode();
                    preset_label = "Custom".to_string();
                }
                if is_key_pressed(KeyCode::F) {
                    config.cycle_fragmentation_mode();
                    preset_label = "Custom".to_string();
                }
                if is_key_pressed(KeyCode::L) {
                    config.cycle_leaderboard_mode();
                    preset_label = "Custom".to_string();
                }
                if is_key_pressed(KeyCode::G) {
                    config.toggle_upgrades();
                    preset_label = "Custom".to_string();
                }
                if is_key_pressed(KeyCode::Y) {
                    preset_index = (preset_index + 1) % presets.len();
                    config = presets[preset_index].clone();
                    preset_label = PRESET_LABELS[preset_index].to_string();
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
                    state = AppState::MainMenu;
                }
            }
            AppState::Leaderboard => {
                menu::draw_leaderboard_menu(&leaderboard);
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
                    state = AppState::MainMenu;
                }
            }
            AppState::Paused => {
                simulation.draw_debug();
                let status = simulation.status();
                menu::draw_score_display(&status);
                if stats_visible {
                    menu::draw_stats_overlay(&config, &status);
                }
                menu::draw_autopilot_status(autopilot_engaged, profile_label(&autopilot_profile));
                if status.game_over {
                    finish_run(
                        &mut simulation,
                        &mut performance_guard,
                        &config,
                        &mut leaderboard,
                        autopilot_engaged,
                        autopilot_profile,
                    );
                    state = AppState::MainMenu;
                    continue;
                }
                draw_text(
                    "PAUSED — press P to resume, Esc to end run",
                    screen_width() / 2.0 - 220.0,
                    screen_height() / 2.0,
                    30.0,
                    WHITE,
                );

                if is_key_pressed(KeyCode::P) {
                    state = AppState::Playing;
                }
                if is_key_pressed(KeyCode::Escape) {
                    finish_run(
                        &mut simulation,
                        &mut performance_guard,
                        &config,
                        &mut leaderboard,
                        autopilot_engaged,
                        autopilot_profile,
                    );
                    state = AppState::MainMenu;
                }
                if is_key_pressed(KeyCode::T) {
                    stats_visible = !stats_visible;
                }
                if is_key_pressed(KeyCode::U) {
                    autopilot_engaged = !autopilot_engaged;
                    set_controller_for_mode(&mut simulation, autopilot_engaged, autopilot_profile);
                }
                if is_key_pressed(KeyCode::I) && autopilot_engaged {
                    autopilot_profile = cycle_profile(autopilot_profile);
                    set_controller_for_mode(&mut simulation, true, autopilot_profile);
                }
            }
            AppState::GameOver => {
                simulation.draw_debug();
                let status = simulation.status();
                menu::draw_score_display(&status);
                if stats_visible {
                    menu::draw_stats_overlay(&config, &status);
                }
                menu::draw_autopilot_status(autopilot_engaged, profile_label(&autopilot_profile));
                menu::draw_game_over(status.score);

                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
                    finish_run(
                        &mut simulation,
                        &mut performance_guard,
                        &config,
                        &mut leaderboard,
                        autopilot_engaged,
                        autopilot_profile,
                    );
                    state = AppState::MainMenu;
                }
                if is_key_pressed(KeyCode::U) {
                    autopilot_engaged = !autopilot_engaged;
                    set_controller_for_mode(&mut simulation, autopilot_engaged, autopilot_profile);
                }
                if is_key_pressed(KeyCode::I) && autopilot_engaged {
                    autopilot_profile = cycle_profile(autopilot_profile);
                    set_controller_for_mode(&mut simulation, true, autopilot_profile);
                }
                if is_key_pressed(KeyCode::T) {
                    stats_visible = !stats_visible;
                }
            }
        }

        next_frame().await;
    }
}

fn finish_run(
    simulation: &mut Simulation,
    performance_guard: &mut PerformanceGuard,
    config: &GameConfig,
    leaderboard: &mut Leaderboard,
    autopilot: bool,
    autopilot_profile: AiProfile,
) {
    let score = simulation.status().score;
    if score > 0 {
        leaderboard.submit("PLAYER", score);
        leaderboard.save();
    }
    *simulation = Simulation::new(config.clone());
    *performance_guard = PerformanceGuard::new();
    set_controller_for_mode(simulation, autopilot, autopilot_profile);
}

fn set_controller_for_mode(
    simulation: &mut Simulation,
    autopilot: bool,
    autopilot_profile: AiProfile,
) {
    if autopilot {
        simulation.set_controller(Box::new(crate::ai::AiController::new(autopilot_profile)));
    } else {
        simulation.set_controller(Box::new(
            crate::controllers::human::HumanController::default(),
        ));
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
