use crate::app::State;
use bevy::prelude::*;

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Name::new("MenuLayout"),
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BorderColor::from(Color::WHITE),
            DespawnOnExit(State::Menu),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Title"),
                Text::new("PONG"),
                TextFont::from_font_size(200.0),
            ));
            parent.spawn((Name::new("AnyKey"), Text::new("Press any key to start")));
        });
}

fn wait_any_key(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<State>>,
) {
    if keyboard_input.get_just_pressed().next().is_some() {
        next_state.set(State::SelectPlayers);
    }
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(State::Menu), setup)
            .add_systems(PreUpdate, wait_any_key.run_if(in_state(State::Menu)));
    }
}
