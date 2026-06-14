use crate::screens::Screen;
use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
};
use rand::{RngExt, rngs::ChaCha8Rng};

const GAME_AREA: Rectangle = Rectangle::new(800.0, 600.0);
const DEFAULT_PADDLE_SPEED: f32 = 300.0;
const DEFAULT_BALL_SPEED: f32 = 100.0;
const BALL_SPEED_STEP: f32 = 0.1;
const BALL_MAX_SPEED: f32 = 1000.0;
const PADDLE_SIZE: Vec2 = Vec2::new(20.0, 100.0);
const BALL_SIZE: Vec2 = Vec2::new(10.0, 10.0);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(Screen = Screen::Game)]
enum GameStatus {
    #[default]
    Running,
    Paused,
}

#[derive(Debug, Resource, Deref, DerefMut)]
struct GameRng(ChaCha8Rng);

impl Default for GameRng {
    fn default() -> Self {
        Self(rand::make_rng())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Resource, Reflect)]
#[reflect(Resource)]
pub enum GameMode {
    #[default]
    OneVsOne,
    OneVsAI,
    AIVsOne,
    AIVsAI,
}

impl GameMode {
    pub fn next(&self) -> Self {
        match self {
            Self::OneVsOne => Self::OneVsAI,
            Self::OneVsAI => Self::AIVsOne,
            Self::AIVsOne => Self::AIVsAI,
            Self::AIVsAI => Self::AIVsAI,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::OneVsOne => Self::OneVsOne,
            Self::OneVsAI => Self::OneVsOne,
            Self::AIVsOne => Self::OneVsAI,
            Self::AIVsAI => Self::AIVsOne,
        }
    }
}

#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct GameScore(u32, u32);

#[derive(Debug, Default, Component, Reflect, Deref, DerefMut)]
struct Direction(Vec2);

#[derive(Debug, Default, Component, Reflect)]
struct Speed(f32);

#[derive(Debug, Default, Component, Reflect, Deref, DerefMut)]
struct Velocity(Vec3);

#[derive(Debug, Default, Component, Reflect, Deref, DerefMut)]
struct CollisionRect(Rectangle);

#[derive(Debug, Default, Component, Reflect, Deref, DerefMut)]
struct CollisionPlane(Plane2d);

#[derive(Debug, Default, Component, Reflect, Deref, DerefMut)]
struct LaunchTimer(Timer);

#[derive(Debug, Component, Reflect)]
struct PaddleKeyboardInput {
    up_key: KeyCode,
    down_key: KeyCode,
}

#[derive(Component, Reflect)]
struct Bot;

#[derive(Component, Reflect)]
struct Paddle;

#[derive(Component, Reflect)]
struct Ball;

#[derive(Component, Reflect)]
struct Wall;

#[derive(Component, Reflect)]
struct RespawnBallArea;

#[derive(Component, Reflect)]
struct ScoreText;

#[derive(Component, Reflect)]
struct NeedsRespawn;

#[derive(Component, Reflect)]
struct NeedsLaunch;

#[derive(Component, Reflect)]
struct Dirty;

fn setup(game_mode: Res<GameMode>, mut commands: Commands) {
    commands.spawn((
        Name::new("MiddleLine"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Sprite::from_color(Color::WHITE, Vec2::new(1.0, GAME_AREA.size().y)),
        DespawnOnExit(Screen::Game),
    ));

    let wall_offset = GAME_AREA.half_size.y;
    commands.spawn((
        Name::new("TopWall"),
        Wall,
        CollisionPlane(Plane2d::new(Vec2::NEG_Y)),
        Transform::from_xyz(0.0, wall_offset, 0.0),
        DespawnOnExit(Screen::Game),
    ));
    commands.spawn((
        Name::new("BottomWall"),
        Wall,
        CollisionPlane(Plane2d::new(Vec2::Y)),
        Transform::from_xyz(0.0, -wall_offset, 0.0),
        DespawnOnExit(Screen::Game),
    ));

    let respawn_offset = GAME_AREA.half_size.x;
    commands.spawn((
        Name::new("LeftRespawn"),
        RespawnBallArea,
        Transform::from_xyz(-respawn_offset, 0.0, 0.0),
        DespawnOnExit(Screen::Game),
    ));
    commands.spawn((
        Name::new("RightRespawn"),
        RespawnBallArea,
        Transform::from_xyz(respawn_offset, 0.0, 0.0),
        DespawnOnExit(Screen::Game),
    ));

    let paddle_offset = GAME_AREA.half_size.x - 20.0;
    let mut left_paddle = commands.spawn((
        Name::new("LeftPaddle"),
        Paddle,
        Speed(DEFAULT_PADDLE_SPEED),
        Direction::default(),
        CollisionRect(Rectangle::from_size(PADDLE_SIZE)),
        Transform::from_xyz(-paddle_offset, 0.0, 0.0),
        Sprite::from_color(Color::WHITE, PADDLE_SIZE),
        DespawnOnExit(Screen::Game),
    ));
    match *game_mode {
        GameMode::OneVsOne | GameMode::OneVsAI => {
            left_paddle.insert(PaddleKeyboardInput {
                up_key: KeyCode::KeyW,
                down_key: KeyCode::KeyS,
            });
        }
        GameMode::AIVsOne | GameMode::AIVsAI => {
            left_paddle.insert(Bot);
        }
    }

    let mut right_paddle = commands.spawn((
        Name::new("RightPaddle"),
        Paddle,
        Speed(DEFAULT_PADDLE_SPEED),
        Direction::default(),
        CollisionRect(Rectangle::from_size(PADDLE_SIZE)),
        Transform::from_xyz(paddle_offset, 0.0, 0.0),
        Sprite::from_color(Color::WHITE, PADDLE_SIZE),
        DespawnOnExit(Screen::Game),
    ));
    match *game_mode {
        GameMode::OneVsOne | GameMode::AIVsOne => {
            right_paddle.insert(PaddleKeyboardInput {
                up_key: KeyCode::ArrowUp,
                down_key: KeyCode::ArrowDown,
            });
        }
        GameMode::OneVsAI | GameMode::AIVsAI => {
            right_paddle.insert(Bot);
        }
    }

    commands.spawn((
        Name::new("Ball"),
        Ball,
        Speed(DEFAULT_BALL_SPEED),
        NeedsRespawn,
        CollisionRect(Rectangle::from_size(BALL_SIZE)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Sprite::from_color(Color::WHITE, BALL_SIZE),
        DespawnOnExit(Screen::Game),
    ));

    commands.spawn((
        Name::new("GameLayout"),
        Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            width: percent(100),
            height: percent(100),
            margin: UiRect {
                top: px(12),
                ..default()
            },
            ..default()
        },
        DespawnOnExit(Screen::Game),
        children![(Name::new("Score"), Text::default(), ScoreText, Dirty)],
    ));
}

fn toggle_pause(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameStatus>>,
    mut next_state: ResMut<NextState<GameStatus>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(match state.get() {
            GameStatus::Running => GameStatus::Paused,
            GameStatus::Paused => GameStatus::Running,
        });
    }
}

fn back_to_menu(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    if keyboard_input.just_pressed(KeyCode::Backspace) {
        next_state.set(Screen::Menu);
    }
}

fn paddle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut inputs: Query<(&PaddleKeyboardInput, &mut Direction), With<Paddle>>,
) {
    for (input, mut direction) in &mut inputs {
        direction.0 = Vec2::ZERO;

        if keyboard_input.pressed(input.up_key) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(input.down_key) {
            direction.y -= 1.0;
        }
    }
}

fn bot_input(
    balls: Query<&Transform, With<Ball>>,
    mut paddles: Query<(&Transform, &mut Direction), (With<Paddle>, With<Bot>)>,
) {
    let area_center = Vec2::ZERO;
    let epsilon = 10.0;

    for ball_transform in &balls {
        let ball_pos = ball_transform.translation.truncate();

        for (paddle_transform, mut direction) in &mut paddles {
            direction.0 = Vec2::ZERO;

            let paddle_pos = paddle_transform.translation.truncate();
            let proximity_x = (paddle_pos.x - area_center.x).abs();
            let close = (ball_pos.x - paddle_pos.x).abs() < proximity_x;

            let target_y = if close { ball_pos.y } else { area_center.y };

            let delta_y = target_y - paddle_pos.y;
            direction.y = if delta_y.abs() > epsilon {
                delta_y.signum()
            } else {
                0.0
            };
        }
    }
}

fn update_score_text(
    game_score: Res<GameScore>,
    mut scores: Query<(Entity, &mut Text), (With<ScoreText>, With<Dirty>)>,
    mut commands: Commands,
) {
    let score = format!("{}   {}", game_score.0, game_score.1);
    for (entity, mut text) in &mut scores {
        *text = Text::new(&score);
        commands.entity(entity).remove::<Dirty>();
    }
}

fn respawn_ball(
    mut balls: Query<(Entity, &mut Transform), (With<Ball>, With<NeedsRespawn>)>,
    mut commands: Commands,
) {
    for (entity, mut transform) in &mut balls {
        transform.translation = Vec3::ZERO;

        commands.entity(entity).remove::<Velocity>();
        commands
            .entity(entity)
            .insert(LaunchTimer(Timer::from_seconds(1.0, TimerMode::Once)));
        commands.entity(entity).remove::<NeedsRespawn>();
    }
}

fn wait_ball_launch_timer(
    time: Res<Time>,
    mut balls: Query<(Entity, &mut LaunchTimer), With<Ball>>,
    mut commands: Commands,
) {
    for (entity, mut timer) in &mut balls {
        if timer.tick(time.delta()).just_finished() {
            commands.entity(entity).insert(NeedsLaunch);
            commands.entity(entity).remove::<LaunchTimer>();
        }
    }
}

fn launch_ball(
    mut rng: ResMut<GameRng>,
    mut balls: Query<(Entity, &mut Speed), (With<Ball>, With<NeedsLaunch>)>,
    mut commands: Commands,
) {
    for (entity, mut speed) in &mut balls {
        let dir_x = if rng.random() { 1.0 } else { -1.0 };
        let dir_y = rng.random::<f32>() * 2.0 - 1.0;
        let new_dir = Vec3::new(dir_x, dir_y, 0.0).normalize();

        commands.entity(entity).insert(Velocity(new_dir));
        speed.0 = DEFAULT_BALL_SPEED;
        commands.entity(entity).remove::<NeedsLaunch>();
    }
}

fn paddle_movement(
    time: Res<Time<Fixed>>,
    walls: Query<(&CollisionPlane, &Transform), (With<Wall>, Without<Paddle>)>,
    mut paddles: Query<(&CollisionRect, &Speed, &Direction, &mut Transform), With<Paddle>>,
) {
    let delta = time.delta().as_secs_f32();
    for (collision_rect, speed, direction, mut transform) in &mut paddles {
        transform.translation.y += direction.y * speed.0 * delta;

        let ray = match direction.y {
            y if y > 0.0 => Ray2d::new(transform.translation.xy(), Dir2::Y),
            y if y < 0.0 => Ray2d::new(transform.translation.xy(), Dir2::NEG_Y),
            _ => continue,
        };

        for (collision_plane, origin) in walls {
            if let Some(hit_distance) =
                ray.intersect_plane(origin.translation.xy(), collision_plane.0)
                && hit_distance < collision_rect.half_size.y
            {
                transform.translation.y =
                    origin.translation.y + collision_rect.half_size.y * -ray.direction.y;
                break;
            }
        }
    }
}

fn ball_movement(
    time: Res<Time<Fixed>>,
    walls: Query<(&CollisionPlane, &Transform), (With<Wall>, Without<Ball>)>,
    paddles: Query<(&CollisionRect, &Transform), (With<Paddle>, Without<Ball>)>,
    mut balls: Query<(&Speed, &CollisionRect, &mut Velocity, &mut Transform), With<Ball>>,
) {
    let delta = time.delta().as_secs_f32();
    for (speed, collision_rect, mut velocity, mut transform) in &mut balls {
        transform.translation += velocity.0 * speed.0 * delta;

        for (wall_collision_plane, wall_transform) in walls {
            let hit_distance = (wall_transform.translation.y - transform.translation.y).abs();
            if hit_distance <= collision_rect.half_size.y {
                let direction = wall_collision_plane.normal.y;
                transform.translation.y =
                    wall_transform.translation.y + collision_rect.half_size.y * direction;
                velocity.0 = velocity.0.reflect(wall_collision_plane.normal.extend(0.0));
                break;
            }
        }

        let ball_aabb = Aabb2d::new(transform.translation.xy(), collision_rect.half_size);
        for (paddle_collision_rect, paddle_transform) in &paddles {
            let paddle_aabb = Aabb2d::new(
                paddle_transform.translation.xy(),
                paddle_collision_rect.half_size,
            );
            if !ball_aabb.intersects(&paddle_aabb) {
                continue;
            }

            let ball_half = ball_aabb.half_size();
            transform.translation.x = if velocity.x > 0.0 {
                paddle_aabb.min.x - ball_half.x
            } else {
                paddle_aabb.max.x + ball_half.x
            };

            let ball_center = ball_aabb.center();
            let paddle_center = paddle_aabb.center();
            let paddle_half = paddle_aabb.half_size();
            let offset = (ball_center.y - paddle_center.y) / paddle_half.y;
            let offset = offset.clamp(-1.0, 1.0);
            let speed = velocity.length();
            let new_dir = Vec3::new(-velocity.x.signum(), offset, 0.0).normalize();
            velocity.0 = new_dir * speed;
        }
    }
}

fn check_score(
    score_text_query: Query<Entity, With<ScoreText>>,
    respawns: Query<&Transform, With<RespawnBallArea>>,
    balls: Query<(Entity, &Transform), With<Ball>>,
    mut game_score: ResMut<GameScore>,
    mut commands: Commands,
) {
    for (entity, transform) in &balls {
        let ball_x = transform.translation.x;
        for respawn_transform in &respawns {
            let respawn_x = respawn_transform.translation.x;
            let ball_out_left = ball_x < respawn_x && respawn_x < 0.0;
            let ball_out_right = ball_x > respawn_x && respawn_x > 0.0;
            let ball_inside_area = !ball_out_left && !ball_out_right;
            if ball_inside_area {
                continue;
            }

            if ball_out_left {
                game_score.1 += 1;
            } else if ball_out_right {
                game_score.0 += 1;
            }

            commands.entity(entity).insert(NeedsRespawn);

            for entity in &score_text_query {
                commands.entity(entity).insert(Dirty);
            }
        }
    }
}

fn update_speed(mut speeds: Query<&mut Speed>) {
    for mut speed in &mut speeds {
        speed.0 += BALL_SPEED_STEP;
        if speed.0 > BALL_MAX_SPEED {
            speed.0 = BALL_MAX_SPEED;
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameRng>();
    app.init_resource::<GameMode>();
    app.init_resource::<GameScore>();
    app.add_sub_state::<GameStatus>();
    app.add_systems(OnEnter(Screen::Game), setup);
    app.add_systems(
        PreUpdate,
        (
            toggle_pause.run_if(in_state(Screen::Game)),
            back_to_menu.run_if(in_state(Screen::Game)),
            (paddle_input, bot_input).run_if(in_state(GameStatus::Running)),
        ),
    );
    app.add_systems(
        Update,
        update_score_text.run_if(in_state(GameStatus::Running)),
    );
    app.add_systems(
        FixedUpdate,
        (
            respawn_ball,
            wait_ball_launch_timer,
            launch_ball,
            paddle_movement,
            ball_movement,
            check_score,
            update_speed,
        )
            .chain()
            .run_if(in_state(GameStatus::Running)),
    );
}
