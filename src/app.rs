use crate::{game::GamePlugin, menu::MenuPlugin, select_players::SelectPlayersPlugin};
use bevy::{prelude::*, window::WindowResolution};

const GAME_TITLE: &str = "PONG";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const CLEAR_COLOR: Color = Color::BLACK;

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
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: GAME_TITLE.to_string(),
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MenuPlugin)
        .add_plugins(SelectPlayersPlugin)
        .add_plugins(GamePlugin)
        .insert_resource(ClearColor(CLEAR_COLOR))
        .init_state::<AppState>()
        .add_systems(Startup, setup);
    }
}
