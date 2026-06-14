use bevy::prelude::*;

mod game;
mod menu;
mod select_players;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum Screen {
    #[default]
    Menu,
    SelectPlayers,
    Game,
}

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((game::plugin, menu::plugin, select_players::plugin));
}
