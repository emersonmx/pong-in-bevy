use crate::{
    game::GamePlugin, menu::MenuPlugin, select_players::SelectPlayersPlugin, states::StatesPlugin,
};
use bevy::{prelude::*, window::WindowResolution};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "PONG".to_string(),
                    resolution: WindowResolution::new(800, 600),
                    ..default()
                }),
                ..default()
            }))
            .add_plugins(StatesPlugin)
            .add_plugins(MenuPlugin)
            .add_plugins(SelectPlayersPlugin)
            .add_plugins(GamePlugin)
            .add_systems(Startup, setup);
    }
}
