use crate::app::AppPlugin;
use bevy::prelude::*;

mod app;
mod game;
mod menu;
mod select_players;

#[cfg(debug_assertions)]
mod debug_plugins;

pub fn run() {
    let mut app = App::new();

    app.add_plugins(AppPlugin);

    #[cfg(debug_assertions)]
    app.add_plugins(debug_plugins::DebugPlugins);

    app.run();
}
