use crate::config::{GameConfig};
use crate::scoreboard::Leaderboard;
use crate::ui::widgets::{draw_menu_box, format_name_with_cursor};

pub fn draw_main_menu() {
    let lines = [
        "ASTEROIDS — SYSTEMS".to_string(),
        "".to_string(),
        "P - Play".to_string(),
        "O - Options".to_string(),
        "L - Leaderboard".to_string(),
        "Esc - Quit".to_string(),
        "".to_string(),
        "".to_string(),
        "Controls:".to_string(),
        "----------------".to_string(),
        "Thrust - W/Up".to_string(),
        "Rotate - Left/Right".to_string(),
        "Fire   - Space -> primary fire".to_string(),
        "Fire   - Shift -> secondary fire".to_string(),
        "".to_string(),
        "U - toggle autopilot".to_string(),
        "P - cycle AI profile when autopilot is enabled / pause otherwise".to_string(),
        "T - toggle stats".to_string(),
        "I - toggle invulnerability".to_string(),
    ];
    draw_menu_box(&lines);
}

pub fn draw_options_menu(config: &GameConfig, preset_label: &str) {
    use crate::config::{CollisionPolicy, FragmentationMode, LeaderboardMode, PhysicsMode};

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

    fn upgrade_label(enabled: bool) -> &'static str {
        if enabled { "On" } else { "Off" }
    }

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
                "{:>2}. {:>8} pts - {}",
                idx + 1,
                entry.score,
                entry.name
            ));
            lines.push(format!(
                "    Shots {}/{}  Acc:{:.1}%",
                entry.stats.shots_fired,
                std::cmp::min(entry.stats.shots_hit, entry.stats.shots_fired),
                entry.stats.accuracy_percent()
            ));
            lines.push(format!(
                "    Hits L/M/S {}/{}/{}  Aliens L/S {}/{}",
                entry.stats.hits_large_asteroid,
                entry.stats.hits_medium_asteroid,
                entry.stats.hits_small_asteroid,
                entry.stats.hits_large_alien,
                entry.stats.hits_small_alien,
            ));
        }
    }
    lines.push("".to_string());
    lines.push("Esc / Enter - Back".to_string());
    draw_menu_box(&lines);
}

pub fn draw_game_over(score: u32, name_input: &str, cursor_pos: usize, name_max: usize) {
    let lines = [
        "GAME OVER".to_string(),
        "".to_string(),
        format!("SCORE {:06}", score),
        "".to_string(),
        format!(
            "Name: {}",
            format_name_with_cursor(name_input, cursor_pos)
        ),
        "ENTER - Submit".to_string(),
        "ESC   - Skip".to_string(),
    ];
    let _ = name_max;
    draw_menu_box(&lines);
}

