pub mod presets {
    use super::{
        AiProfile, Budgets, CollisionPolicy, FragmentationMode, GameConfig, LeaderboardMode,
        PhysicsMode, PlayerControllerMode,
    };

    pub fn default_presets() -> Vec<GameConfig> {
        vec![classic(), arcade_upgrades(), ai_autopilot()]
    }

    fn classic() -> GameConfig {
        GameConfig {
            player_controller: PlayerControllerMode::Human,
            leaderboard_mode: LeaderboardMode::LocalTop10,
            budgets: Budgets::classic(),
            physics_mode: PhysicsMode::Arcade,
            fragmentation_mode: FragmentationMode::ClassicSplit,
            upgrades_enabled: false,
            collision_policy: CollisionPolicy::PlayerOnly,
        }
    }

    fn arcade_upgrades() -> GameConfig {
        GameConfig {
            player_controller: PlayerControllerMode::Human,
            leaderboard_mode: LeaderboardMode::LocalTop10,
            budgets: Budgets::arcade(),
            physics_mode: PhysicsMode::Arcade,
            fragmentation_mode: FragmentationMode::ClassicSplit,
            upgrades_enabled: true,
            collision_policy: CollisionPolicy::PlayerOnly,
        }
    }

    fn ai_autopilot() -> GameConfig {
        GameConfig {
            player_controller: PlayerControllerMode::Ai {
                profile: AiProfile::Balanced,
            },
            leaderboard_mode: LeaderboardMode::LocalTop10,
            budgets: Budgets::classic(),
            physics_mode: PhysicsMode::Arcade,
            fragmentation_mode: FragmentationMode::ClassicSplit,
            upgrades_enabled: false,
            collision_policy: CollisionPolicy::PlayerOnly,
        }
    }
}

#[derive(Clone)]
pub struct GameConfig {
    pub player_controller: PlayerControllerMode,
    pub leaderboard_mode: LeaderboardMode,
    pub budgets: Budgets,
    pub physics_mode: PhysicsMode,
    pub fragmentation_mode: FragmentationMode,
    pub upgrades_enabled: bool,
    pub collision_policy: CollisionPolicy,
}

#[derive(Clone)]
pub struct Budgets {
    pub max_bodies: usize,
    pub frag_event_cap: usize,
    pub debris_ttl_ms: u64,
    pub big_collision_radius: f32,
    pub v_max: usize,
}

impl Budgets {
    pub fn classic() -> Self {
        Self {
            max_bodies: 800,
            frag_event_cap: 4,
            debris_ttl_ms: 900,
            big_collision_radius: 32.0,
            v_max: 24,
        }
    }

    pub fn arcade() -> Self {
        Self {
            max_bodies: 900,
            frag_event_cap: 4,
            debris_ttl_ms: 900,
            big_collision_radius: 32.0,
            v_max: 24,
        }
    }
}

#[derive(Clone)]
pub enum PlayerControllerMode {
    Human,
    Ai { profile: AiProfile },
}

#[derive(Clone, Copy, Debug)]
pub enum AiProfile {
    Casual,
    Balanced,
    Veteran,
}

#[derive(Clone)]
pub enum LeaderboardMode {
    Off,
    LocalTop10,
}

#[derive(Clone)]
pub enum PhysicsMode {
    Off,
    Arcade,
    Lite,
}

#[derive(Clone)]
pub enum FragmentationMode {
    Off,
    ClassicSplit,
    SliceOnly,
    Explode,
    Full,
}

#[derive(Clone)]
pub enum CollisionPolicy {
    PlayerOnly,
    BigOnly,
    Full,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            player_controller: PlayerControllerMode::Human,
            leaderboard_mode: LeaderboardMode::LocalTop10,
            budgets: Budgets::classic(),
            physics_mode: PhysicsMode::Arcade,
            fragmentation_mode: FragmentationMode::ClassicSplit,
            upgrades_enabled: false,
            collision_policy: CollisionPolicy::PlayerOnly,
        }
    }
}

impl GameConfig {
    pub fn cycle_physics_mode(&mut self) {
        self.physics_mode = match self.physics_mode {
            PhysicsMode::Off => PhysicsMode::Arcade,
            PhysicsMode::Arcade => PhysicsMode::Lite,
            PhysicsMode::Lite => PhysicsMode::Off,
        };
    }

    pub fn cycle_collision_policy(&mut self) {
        self.collision_policy = match self.collision_policy {
            CollisionPolicy::PlayerOnly => CollisionPolicy::BigOnly,
            CollisionPolicy::BigOnly => CollisionPolicy::Full,
            CollisionPolicy::Full => CollisionPolicy::PlayerOnly,
        };
    }

    pub fn cycle_fragmentation_mode(&mut self) {
        let next = match self.fragmentation_mode.clone() {
            FragmentationMode::Off => FragmentationMode::ClassicSplit,
            FragmentationMode::ClassicSplit => FragmentationMode::SliceOnly,
            FragmentationMode::SliceOnly => FragmentationMode::Explode,
            FragmentationMode::Explode => FragmentationMode::Full,
            FragmentationMode::Full => FragmentationMode::Off,
        };
        self.fragmentation_mode = next;
    }

    pub fn cycle_leaderboard_mode(&mut self) {
        let next = match self.leaderboard_mode.clone() {
            LeaderboardMode::Off => LeaderboardMode::LocalTop10,
            LeaderboardMode::LocalTop10 => LeaderboardMode::Off,
        };
        self.leaderboard_mode = next;
    }

    pub fn toggle_upgrades(&mut self) {
        self.upgrades_enabled = !self.upgrades_enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_presets_cover_play_modes() {
        let presets = presets::default_presets();
        assert_eq!(presets.len(), 3);
        assert!(matches!(
            presets[0].player_controller,
            PlayerControllerMode::Human
        ));
        assert!(matches!(
            presets[2].player_controller,
            PlayerControllerMode::Ai {
                profile: AiProfile::Balanced
            }
        ));
    }

    #[test]
    fn budgets_have_expected_values() {
        let classic = Budgets::classic();
        let arcade = Budgets::arcade();
        assert_eq!(classic.max_bodies, 800);
        assert_eq!(arcade.max_bodies, 900);
        assert!(classic.max_bodies < arcade.max_bodies);
    }

    #[test]
    fn physics_cycle_wraps() {
        let mut config = GameConfig::default();
        config.physics_mode = PhysicsMode::Lite;
        config.cycle_physics_mode();
        assert!(matches!(config.physics_mode, PhysicsMode::Off));
    }

    #[test]
    fn collision_cycle_wraps() {
        let mut config = GameConfig::default();
        config.collision_policy = CollisionPolicy::Full;
        config.cycle_collision_policy();
        assert!(matches!(
            config.collision_policy,
            CollisionPolicy::PlayerOnly
        ));
    }
}
