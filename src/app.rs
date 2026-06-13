use crate::{game::GamePlugin, menu::MenuPlugin, select_players::SelectPlayersPlugin};
use bevy::{prelude::*, window::WindowResolution};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    SelectPlayers,
    Game,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .init_state::<AppState>()
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "PONG".to_string(),
                    resolution: WindowResolution::new(800, 600),
                    ..default()
                }),
                ..default()
            }))
            .add_plugins(MenuPlugin)
            .add_plugins(SelectPlayersPlugin)
            .add_plugins(GamePlugin)
            .add_systems(Startup, setup);
    }
}
