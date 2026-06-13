use crate::{game::GameMode, states::AppState};
use bevy::prelude::*;

#[derive(Debug, Default, Component, Deref, DerefMut, Reflect)]
struct GameModeOption(GameMode);

#[derive(Component, Reflect)]
struct Dirty;

fn setup(mut game_mode: ResMut<GameMode>, mut commands: Commands) {
    *game_mode = default();

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
                        ("OneVsOne", "1", "vs", "1", GameMode::OneVsOne),
                        ("OneVsAI", "1", "vs", "AI", GameMode::OneVsAI),
                        ("AIVsOne", "AI", "vs", "1", GameMode::AIVsOne),
                        ("AIVsAI", "AI", "vs", "AI", GameMode::AIVsAI),
                    ];

                    for (name, left, middle, right, mode) in options {
                        let mut entity = parent.spawn((
                            Name::new(name),
                            layout.clone(),
                            GameModeOption(mode),
                            Dirty,
                        ));
                        entity.with_children(|parent| {
                            parent.spawn((item.clone(), Text::new(left)));
                            parent.spawn((item.clone(), Text::new(middle)));
                            parent.spawn((item.clone(), Text::new(right)));
                        });
                    }
                });
        });
}

fn select_option(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut game_mode: ResMut<GameMode>,
    mut next_state: ResMut<NextState<AppState>>,
    mode_options: Query<Entity, (With<GameModeOption>, Without<Dirty>)>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Game);
        return;
    }

    let up_pressed = keyboard_input.just_pressed(KeyCode::ArrowUp);
    let down_pressed = keyboard_input.just_pressed(KeyCode::ArrowDown);
    let is_dirty = up_pressed || down_pressed;

    if up_pressed {
        *game_mode = game_mode.previous();
    }
    if down_pressed {
        *game_mode = game_mode.next();
    }

    if is_dirty {
        for entity in &mode_options {
            commands.entity(entity).insert(Dirty);
        }
    }
}

fn update_selection(
    mut commands: Commands,
    game_mode: ResMut<GameMode>,
    dirty_query: Query<(Entity, &Children, &GameModeOption), With<Dirty>>,
) {
    for (entity, children, game_mode_option) in &dirty_query {
        commands.entity(entity).remove::<BackgroundColor>();
        commands.entity(entity).remove::<Dirty>();

        for &child in children {
            commands.entity(child).insert(TextColor::from(Color::WHITE));
        }

        if *game_mode == game_mode_option.0 {
            commands
                .entity(entity)
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
        app.add_systems(OnEnter(AppState::SelectPlayers), setup)
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
