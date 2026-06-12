use crate::game::GamePlugin;
use bevy::{prelude::*, window::WindowResolution};
use debug_plugins::DebugPlugins;

mod debug_plugins;
mod game;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "PONG".to_string(),
                resolution: WindowResolution::new(800, 600),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(DebugPlugins)
        .add_plugins(GamePlugin)
        .run();
}
