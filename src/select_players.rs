use crate::{
    game::{Game, Mode},
    states::AppState,
};
use bevy::prelude::*;

#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
struct SelectPlayers {
    options: Vec<Entity>,
    selected_mode: Mode,
}

#[derive(Component, Reflect)]
struct Dirty;

fn setup(
    mut game: ResMut<Game>,
    mut select_players: ResMut<SelectPlayers>,
    mut commands: Commands,
) {
    *game = default();
    *select_players = default();

    commands
        .spawn((
            Name::new("SelectPlayersLayout"),
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            DespawnOnExit(AppState::SelectPlayers),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Title"),
                Text::new("Select Players"),
                TextFont::from_font_size(64.0),
            ));
            parent
                .spawn((
                    Name::new("Options"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: percent(20),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    let layout = Node {
                        justify_content: JustifyContent::SpaceAround,
                        border: UiRect::all(px(1)),
                        ..default()
                    };
                    let item = Node {
                        width: percent(10),
                        ..default()
                    };
                    let options = [
                        ("OneVsOne", "1", "vs", "1"),
                        ("OneVsAI", "1", "vs", "AI"),
                        ("AIVsOne", "AI", "vs", "1"),
                        ("AIVsAI", "AI", "vs", "AI"),
                    ];

                    for (i, &(name, left, middle, right)) in options.iter().enumerate() {
                        let mut entity = parent.spawn((Name::new(name), layout.clone(), Dirty));
                        entity.with_children(|parent| {
                            parent.spawn((item.clone(), Text::new(left)));
                            parent.spawn((item.clone(), Text::new(middle)));
                            parent.spawn((item.clone(), Text::new(right)));
                        });
                        select_players.options.push(entity.id());
                        if i == 0 {
                            select_players.selected_mode = Mode::OneVsOne;
                        }
                    }
                });
        });
}

fn select_option(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut select_players: ResMut<SelectPlayers>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        game.mode = select_players.selected_mode.clone();
        next_state.set(AppState::Game);
        return;
    }

    let up_pressed = keyboard_input.just_pressed(KeyCode::ArrowUp);
    let down_pressed = keyboard_input.just_pressed(KeyCode::ArrowDown);
    let is_dirty = up_pressed || down_pressed;

    if select_players.selected_mode == Mode::OneVsOne && up_pressed {
        return;
    }

    if select_players.selected_mode == Mode::AIVsAI && down_pressed {
        return;
    }

    if up_pressed {
        select_players.selected_mode = select_players.selected_mode.previous_mode();
    }
    if down_pressed {
        select_players.selected_mode = select_players.selected_mode.next_mode();
    }

    if is_dirty {
        for entity in &select_players.options {
            commands.entity(*entity).insert(Dirty);
        }
    }
}

fn update_selection(
    mut commands: Commands,
    select_players: ResMut<SelectPlayers>,
    dirty_query: Query<(Entity, &Children), With<Dirty>>,
) {
    let selected_option = select_players.options[select_players.selected_mode.clone() as usize];
    for (entity, children) in &dirty_query {
        commands.entity(entity).remove::<BackgroundColor>();
        commands.entity(entity).remove::<Dirty>();

        for &child in children {
            commands.entity(child).insert(TextColor::from(Color::WHITE));
        }

        if entity == selected_option {
            commands
                .entity(selected_option)
                .insert(BackgroundColor::from(Color::WHITE));
            for &child in children {
                commands.entity(child).insert(TextColor::from(Color::BLACK));
            }
        }
    }
}

pub struct SelectPlayersPlugin;

impl Plugin for SelectPlayersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectPlayers>()
            .add_systems(OnEnter(AppState::SelectPlayers), setup)
            .add_systems(
                PreUpdate,
                select_option.run_if(in_state(AppState::SelectPlayers)),
            )
            .add_systems(
                Update,
                update_selection.run_if(in_state(AppState::SelectPlayers)),
            );
    }
}
