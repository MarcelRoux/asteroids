#[derive(PartialEq, Eq)]
pub enum AppState {
    MainMenu,
    Options,
    Leaderboard,
    Playing,
    Paused,
    GameOver,
}
