pub mod menu {
    use crate::config::{
        CollisionPolicy, FragmentationMode, GameConfig, LeaderboardMode, PhysicsMode,
        PlayerControllerMode,
    };
    use crate::scoreboard::Leaderboard;
    use crate::simulation::SimulationStatus;
    use macroquad::prelude::{BLACK, Color, WHITE, draw_rectangle, draw_text, screen_width};

    const OVERLAY_WIDTH: f32 = 240.0;
    const OVERLAY_MARGIN: f32 = 16.0;

    pub fn draw_stats_overlay(config: &GameConfig, status: &SimulationStatus) {
        let mut lines = vec![
            format!("FPS: {:.1}", status.fps),
            format!("Frame: {}", status.frame),
            format!("Score: {}", status.score),
            format!("Asteroids: {}", status.asteroid_count),
            format!("Bullets: {}", status.bullet_count),
            format!("Bodies: {}", status.active_bodies),
            format!("Primary CD: {:.2}s", status.primary_cooldown),
            format!("Secondary CD: {:.2}s", status.secondary_cooldown),
            format!(
                "Controller: {}",
                controller_label(&config.player_controller)
            ),
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

        let height = 18.0 * (lines.len() as f32) + 16.0;
        let x = screen_width() - OVERLAY_WIDTH - OVERLAY_MARGIN;
        draw_rectangle(
            x,
            OVERLAY_MARGIN,
            OVERLAY_WIDTH,
            height,
            Color::new(0.0, 0.0, 0.0, 0.6),
        );

        let mut y = OVERLAY_MARGIN + 16.0;
        for line in &lines {
            draw_text(line, x + 8.0, y, 16.0, WHITE);
            y += 18.0;
        }
    }

    pub fn draw_score_display(status: &SimulationStatus) {
        let text = format!("SCORE {:06}   LIVES: {}", status.score, status.lives);
        let x = screen_width() / 2.0 - 170.0;
        draw_text(&text, x, 42.0, 32.0, WHITE);
    }

    pub fn draw_game_over(score: u32) {
        let msg = format!("GAME OVER  SCORE {:06}", score);
        draw_menu_box(&[
            "GAME OVER".to_string(),
            "".to_string(),
            msg,
            "".to_string(),
            "ENTER / ESC - RETURN TO MENU".to_string(),
        ]);
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

    pub fn draw_main_menu() {
        let lines = [
            "ASTEROIDS — SYSTEMS".to_string(),
            "".to_string(),
            "P - Play".to_string(),
            "O - Options".to_string(),
            "L - Leaderboard".to_string(),
            "Esc - Quit".to_string(),
            "".to_string(),
            "Controls:".to_string(),
            "W/Up - thrust, A/D or ←/→ - rotate".to_string(),
            "Space - primary fire, Shift - secondary fire".to_string(),
            "U - toggle autopilot".to_string(),
            "I - cycle autopilot profile".to_string(),
            "P - pause/resume once playing, T - toggle stats".to_string(),
        ];
        draw_menu_box(&lines);
    }

    pub fn draw_options_menu(config: &GameConfig, preset_label: &str) {
        let lines = [
            "OPTIONS".to_string(),
            "".to_string(),
            format!(
                "C - Collision Policy: {}",
                collision_label(&config.collision_policy)
            ),
            format!("K - Physics Mode: {}", physics_label(&config.physics_mode)),
            format!(
                "F - Fragmentation Mode: {}",
                fragmentation_label(&config.fragmentation_mode)
            ),
            format!(
                "L - Leaderboard Mode: {}",
                leaderboard_label(&config.leaderboard_mode)
            ),
            format!("G - Upgrades: {}", upgrade_label(config.upgrades_enabled)),
            format!("Y - Preset: {}", preset_label),
            "".to_string(),
            "Enter / Esc - Back".to_string(),
        ];
        draw_menu_box(&lines);
    }

    pub fn draw_leaderboard_menu(leaderboard: &Leaderboard) {
        let mut lines = vec!["LEADERBOARD".to_string(), "".to_string()];
        if leaderboard.entries().is_empty() {
            lines.push("No runs recorded yet.".to_string());
        } else {
            for (idx, entry) in leaderboard.entries().iter().enumerate() {
                lines.push(format!(
                    "{:>2}. {:>6} pts - {}",
                    idx + 1,
                    entry.score,
                    entry.name
                ));
            }
        }
        lines.push("".to_string());
        lines.push("Esc / Enter - Back".to_string());
        draw_menu_box(&lines);
    }

    fn draw_menu_box(lines: &[String]) {
        let width = 460.0;
        let height = 24.0 * (lines.len() as f32) + 32.0;
        let x = 40.0;
        let y = 60.0;
        draw_rectangle(x, y, width, height, BLACK);
        let mut offset = y + 30.0;
        for line in lines {
            draw_text(line, x + 12.0, offset, 24.0, WHITE);
            offset += 26.0;
        }
    }

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
}
