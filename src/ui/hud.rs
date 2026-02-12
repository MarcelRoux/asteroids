use crate::config::GameConfig;
use crate::render::shapes::{draw_shape, ship_lines};
use crate::simulation::SimulationStatus;
use macroquad::prelude::{Color, Vec2, WHITE, draw_rectangle, draw_text, screen_height, screen_width};
use macroquad::text::measure_text;
use std::f32::consts::PI;

const OVERLAY_WIDTH: f32 = 240.0;
const OVERLAY_MARGIN: f32 = 16.0;
const LIFE_ICON_SCALE: f32 = 10.0;
const LIFE_ICON_SPACING: f32 = 28.0;
const LIFE_ICON_MARGIN: f32 = 20.0;
const LIFE_ICON_Y: f32 = 32.0;
const LIFE_ICON_STROKE: f32 = 1.6;
const SCORE_FONT_SIZE: f32 = 32.0;

pub fn draw_stats_overlay(config: &GameConfig, status: &SimulationStatus) {
    use crate::config::{
        CollisionPolicy, FragmentationMode, LeaderboardMode, PhysicsMode, PlayerControllerMode,
    };

    fn fragmentation_label(mode: &FragmentationMode) -> &'static str {
        match mode {
            FragmentationMode::Off => "Off",
            FragmentationMode::ClassicSplit => "ClassicSplit",
            FragmentationMode::SliceOnly => "SliceOnly",
            FragmentationMode::Explode => "Explode",
            FragmentationMode::Full => "Full",
        }
    }

    fn leaderboard_label(mode: &LeaderboardMode) -> &'static str {
        match mode {
            LeaderboardMode::Off => "Off",
            LeaderboardMode::LocalTop10 => "LocalTop10",
        }
    }

    fn upgrade_label(enabled: bool) -> &'static str {
        if enabled { "On" } else { "Off" }
    }

    fn controller_label(mode: &PlayerControllerMode) -> &'static str {
        match mode {
            PlayerControllerMode::Human => "Human",
            PlayerControllerMode::Ai { .. } => "AI",
        }
    }

    fn physics_label(mode: &PhysicsMode) -> &'static str {
        match mode {
            PhysicsMode::Off => "Off",
            PhysicsMode::Arcade => "Arcade",
            PhysicsMode::Lite => "Lite",
        }
    }

    fn collision_label(mode: &CollisionPolicy) -> &'static str {
        match mode {
            CollisionPolicy::PlayerOnly => "PlayerOnly",
            CollisionPolicy::BigOnly => "BigOnly",
            CollisionPolicy::Full => "Full",
        }
    }

    let mut lines = vec![
        format!("FPS: {:.1}", status.fps),
        format!("Frame: {}", status.frame),
        format!("Score: {}", status.score),
        format!("Asteroids: {}", status.asteroid_count),
        format!("Bullets: {}", status.bullet_count),
        format!("Bodies: {}", status.active_bodies),
        format!("Primary CD: {:.2}s", status.primary_cooldown),
        format!("Secondary CD: {:.2}s", status.secondary_cooldown),
        format!("Controller: {}", controller_label(&config.player_controller)),
        format!("Physics: {}", physics_label(&config.physics_mode)),
        format!("Collision: {}", collision_label(&config.collision_policy)),
    ];
    lines.push(format!(
        "Fragmentation: {}",
        fragmentation_label(&config.fragmentation_mode)
    ));
    lines.push(format!(
        "Leaderboard: {}",
        leaderboard_label(&config.leaderboard_mode)
    ));
    lines.push(format!(
        "Upgrades: {}",
        upgrade_label(config.upgrades_enabled)
    ));
    lines.push(format!(
        "Budgets: max={} frag={} ttl={}ms radius={:.1} v_max={}",
        config.budgets.max_bodies,
        config.budgets.frag_event_cap,
        config.budgets.debris_ttl_ms,
        config.budgets.big_collision_radius,
        config.budgets.v_max,
    ));
    lines.push(format!("Shots Fired: {}", status.run_stats.shots_fired));
    lines.push(format!("Shots Hit: {}", status.run_stats.shots_hit));
    lines.push(format!("Accuracy: {:.1}%", status.run_stats.accuracy_percent()));
    lines.push(format!(
        "Hits (Ast L/M/S): {}/{}/{}",
        status.run_stats.hits_large_asteroid,
        status.run_stats.hits_medium_asteroid,
        status.run_stats.hits_small_asteroid,
    ));
    lines.push(format!(
        "Hits (Aliens L/S): {}/{}",
        status.run_stats.hits_large_alien, status.run_stats.hits_small_alien,
    ));
    lines.push(format!(
        "Invuln: {}",
        if status.invulnerability_enabled { "On" } else { "Off" }
    ));

    let height = 18.0 * (lines.len() as f32) + 16.0;
    let x = OVERLAY_MARGIN;
    let y = screen_height() - OVERLAY_MARGIN - height;
    draw_rectangle(x, y, OVERLAY_WIDTH, height, Color::new(0.0, 0.0, 0.0, 0.6));

    let mut offset = y + 16.0;
    for line in &lines {
        draw_text(line, x + 8.0, offset, 16.0, WHITE);
        offset += 18.0;
    }
}

pub fn draw_score_display(status: &SimulationStatus) {
    let text = format!("{:06}", status.score);
    let metrics = measure_text(&text, None, SCORE_FONT_SIZE as u16, 1.0);
    let x = screen_width() - OVERLAY_MARGIN - metrics.width;
    draw_text(&text, x, 42.0, SCORE_FONT_SIZE, WHITE);
    draw_life_icons(status.lives);
}

fn draw_life_icons(lives: u32) {
    let segments = ship_lines(LIFE_ICON_SCALE);
    for i in 0..lives {
        let x = LIFE_ICON_MARGIN + i as f32 * LIFE_ICON_SPACING;
        let position = Vec2::new(x, LIFE_ICON_Y);
        draw_shape(&segments, position, PI, LIFE_ICON_STROKE, WHITE, false);
    }
}

pub fn draw_autopilot_status(engaged: bool, profile: &str) {
    if engaged {
        draw_text(
            &format!("Autopilot: Engaged ({})", profile),
            screen_width() / 2.0 - 140.0,
            78.0,
            20.0,
            Color::new(0.5, 1.0, 0.4, 1.0),
        );
    }
}

