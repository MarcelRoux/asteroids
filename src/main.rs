mod ai;
mod app;
mod config;
mod controllers;
mod eval;
mod render;
mod scoreboard;
mod simulation;
mod stats;
mod ui;

use app::App;

#[macroquad::main("Asteroids Systems")]
async fn main() {
    let mut app = App::new();
    while app.tick().await {}
}
