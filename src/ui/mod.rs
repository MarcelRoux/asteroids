pub mod hud;
pub mod screens;
pub mod widgets;

// Backward-compatible facade: keep `crate::ui::menu::*` call sites stable.
pub mod menu {
    pub use crate::ui::hud::{draw_autopilot_status, draw_score_display, draw_stats_overlay};
    pub use crate::ui::screens::{
        draw_game_over, draw_leaderboard_menu, draw_main_menu, draw_options_menu,
    };
}

