use crate::{game::GamePlugin, menu::MenuPlugin, states::StatePlugin};
use bevy::{prelude::*, window::WindowResolution};
use debug_plugins::DebugPlugins;

mod debug_plugins;
mod game;
mod menu;
mod states;

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
        .add_plugins(StatePlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(GamePlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
