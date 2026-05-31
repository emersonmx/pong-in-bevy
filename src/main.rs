use bevy::{prelude::*, window::WindowResolution};
use rand::{RngExt, rngs::ChaCha8Rng};

#[derive(Debug, Default, Copy, Clone)]
struct Aabb {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

impl Aabb {
    fn new(center: Vec2, size: Vec2) -> Self {
        let half_size = size / 2.0;
        Self {
            left: center.x - half_size.x,
            right: center.x + half_size.x,
            top: center.y + half_size.y,
            bottom: center.y - half_size.y,
        }
    }

    fn center(&self) -> Vec2 {
        Vec2::new(
            (self.left + self.right) / 2.0,
            (self.top + self.bottom) / 2.0,
        )
    }

    fn size(&self) -> Vec2 {
        Vec2::new(self.right - self.left, self.top - self.bottom)
    }

    fn half_size(&self) -> Vec2 {
        self.size() / 2.0
    }

    fn intersects(&self, other: &Aabb) -> bool {
        self.left < other.right
            && self.right > other.left
            && self.top > other.bottom
            && self.bottom < other.top
    }
}

#[derive(Debug, Default, Component, Deref, DerefMut)]
struct Direction(Vec2);

#[derive(Debug, Default, Component, Deref, DerefMut)]
struct Speed(f32);

#[derive(Component)]
struct SpeedStep(f32);

#[derive(Debug, Default, Component, Deref, DerefMut)]
struct Velocity(Vec3);

#[derive(Debug, Component, Deref, DerefMut)]
struct CollisionRect(Aabb);

#[derive(Debug, Default, Component, Deref, DerefMut)]
struct LaunchTimer(Timer);

#[derive(Debug, Component)]
struct PaddleKeyboardInput {
    up_key: KeyCode,
    down_key: KeyCode,
}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct NeedsReset;

#[derive(Component)]
struct NeedsLaunch;

#[derive(Component)]
struct Dirty;

#[derive(Debug, Resource)]
struct Game {
    rng: ChaCha8Rng,
    area: Aabb,
    score: (u32, u32),
    default_ball_speed: f32,
    ball_max_speed: f32,
}

fn setup(game: Res<Game>, mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Name::new("middle line"),
        Transform::from_translation(game.area.center().extend(0.0)),
        Sprite::from_color(Color::WHITE, Vec2::new(1.0, 600.0)),
    ));

    let paddle_size = Vec2::new(20.0, 100.0);
    let left_position = Vec3::new(game.area.left + paddle_size.x, 0.0, 0.0);
    commands.spawn((
        Name::new("left paddle"),
        Paddle,
        PaddleKeyboardInput {
            up_key: KeyCode::KeyW,
            down_key: KeyCode::KeyS,
        },
        Speed(300.0),
        Direction::default(),
        CollisionRect(Aabb::new(left_position.truncate(), paddle_size)),
        Transform::from_translation(left_position),
        Sprite::from_color(Color::WHITE, paddle_size),
    ));

    let right_position = Vec3::new(game.area.right - paddle_size.x, 0.0, 0.0);
    commands.spawn((
        Name::new("right paddle"),
        Paddle,
        PaddleKeyboardInput {
            up_key: KeyCode::ArrowUp,
            down_key: KeyCode::ArrowDown,
        },
        Speed(300.0),
        Direction::default(),
        CollisionRect(Aabb::new(right_position.truncate(), paddle_size)),
        Transform::from_translation(right_position),
        Sprite::from_color(Color::WHITE, paddle_size),
    ));

    let ball_size = Vec2::new(10.0, 10.0);
    let ball_position = game.area.center();
    commands.spawn((
        Name::new("ball"),
        Ball,
        Speed(game.default_ball_speed),
        SpeedStep(0.1),
        NeedsReset,
        CollisionRect(Aabb::new(ball_position, ball_size)),
        Transform::from_translation(ball_position.extend(0.0)),
        Sprite::from_color(Color::WHITE, ball_size),
    ));

    commands
        .spawn((Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            width: percent(100),
            height: percent(100),
            margin: UiRect {
                top: px(12),
                ..default()
            },
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((Name::new("score"), Text::default(), ScoreText, Dirty));
        });
}

fn paddle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&PaddleKeyboardInput, &mut Direction), With<Paddle>>,
) {
    for (input, mut direction) in query.iter_mut() {
        direction.x = 0.0;
        direction.y = 0.0;

        if keyboard_input.pressed(input.up_key) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(input.down_key) {
            direction.y -= 1.0;
        }
    }
}

fn close_on_esc(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_exit_messages: MessageWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_messages.write(AppExit::Success);
    }
}

fn update_score_text(
    game: Res<Game>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Text), (With<ScoreText>, With<Dirty>)>,
) {
    let score = format!("{}   {}", game.score.0, game.score.1);
    for (entity, mut text) in query.iter_mut() {
        *text = Text::new(&score);
        commands.entity(entity).remove::<Dirty>();
    }
}

fn reset_ball(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), (With<Ball>, With<NeedsReset>)>,
    game: Res<Game>,
) {
    for (entity, mut transform) in query.iter_mut() {
        transform.translation = game.area.center().extend(0.0);

        commands.entity(entity).remove::<Velocity>();
        commands
            .entity(entity)
            .insert(LaunchTimer(Timer::from_seconds(1.0, TimerMode::Once)));
        commands.entity(entity).remove::<NeedsReset>();
    }
}

fn wait_ball_launch_timer(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut LaunchTimer), With<Ball>>,
) {
    for (entity, mut timer) in query.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            commands.entity(entity).insert(NeedsLaunch);
            commands.entity(entity).remove::<LaunchTimer>();
        }
    }
}

fn launch_ball(
    mut game: ResMut<Game>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Speed), (With<Ball>, With<NeedsLaunch>)>,
) {
    for (entity, mut speed) in query.iter_mut() {
        let dir_x = if game.rng.random() { 1.0 } else { -1.0 };
        let dir_y = game.rng.random::<f32>() * 2.0 - 1.0;
        let new_dir = Vec3::new(dir_x, dir_y, 0.0).normalize();

        commands.entity(entity).insert(Velocity(new_dir));
        speed.0 = game.default_ball_speed;
        commands.entity(entity).remove::<NeedsLaunch>();
    }
}

fn move_paddle(
    time: Res<Time<Fixed>>,
    mut query: Query<(&Speed, &Direction, &mut Transform), With<Paddle>>,
) {
    let delta = time.delta().as_secs_f32();
    for (speed, direction, mut transform) in query.iter_mut() {
        transform.translation.y += direction.y * speed.0 * delta;
    }
}

fn move_ball(
    time: Res<Time<Fixed>>,
    game: Res<Game>,
    mut query: Query<(&Speed, &CollisionRect, &mut Velocity, &mut Transform), With<Ball>>,
) {
    let delta = time.delta().as_secs_f32();
    let game_area = game.area;
    for (speed, collision_rect, mut velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0 * speed.0 * delta;

        let pos = transform.translation.truncate();
        let half_size = collision_rect.half_size();
        if pos.y + half_size.y > game_area.top || pos.y - half_size.y < game_area.bottom {
            velocity.y = -velocity.y;
        }
    }
}

fn clamp_position_to_game_area_top_and_bottom(
    game: Res<Game>,
    mut query: Query<(&CollisionRect, &mut Transform)>,
) {
    let game_area = game.area;
    for (collision_rect, mut transform) in query.iter_mut() {
        let pos = transform.translation.truncate();
        let half_size = collision_rect.half_size();
        transform.translation.y = pos
            .y
            .clamp(game_area.bottom + half_size.y, game_area.top - half_size.y);
    }
}

fn bounce_ball_on_paddle(
    paddle_query: Query<&CollisionRect, With<Paddle>>,
    mut ball_query: Query<(&CollisionRect, &mut Velocity, &mut Transform), With<Ball>>,
) {
    for (ball_collision_rect, mut ball_velocity, mut transform) in ball_query.iter_mut() {
        for paddle_collision_rect in paddle_query.iter() {
            if ball_collision_rect.intersects(paddle_collision_rect) {
                let ball_center = ball_collision_rect.center();
                let ball_half = ball_collision_rect.half_size();
                let paddle_center = paddle_collision_rect.center();
                let paddle_half = paddle_collision_rect.half_size();

                let dx = ball_center.x - paddle_center.x;
                let dy = ball_center.y - paddle_center.y;

                let overlap_x = paddle_half.x + ball_half.x - dx.abs();
                let overlap_y = paddle_half.y + ball_half.y - dy.abs();

                if overlap_x < overlap_y {
                    transform.translation.x = if dx > 0.0 {
                        paddle_collision_rect.right + ball_half.x
                    } else {
                        paddle_collision_rect.left - ball_half.x
                    };

                    let offset = (ball_center.y - paddle_center.y) / paddle_half.y;
                    let offset = offset.clamp(-1.0, 1.0);
                    let speed = ball_velocity.length();
                    let new_dir = Vec2::new(-ball_velocity.x.signum(), offset).normalize();
                    ball_velocity.x = new_dir.x * speed;
                    ball_velocity.y = new_dir.y * speed;
                } else {
                    transform.translation.y = if dy > 0.0 {
                        paddle_collision_rect.top + ball_half.y
                    } else {
                        paddle_collision_rect.bottom - ball_half.y
                    };

                    ball_velocity.y = -ball_velocity.y;
                }
            }
        }
    }
}

fn update_collision_rect(mut query: Query<(&Transform, &mut CollisionRect)>) {
    for (transform, mut collision_rect) in query.iter_mut() {
        let center = transform.translation.truncate();
        let size = collision_rect.size();
        *collision_rect = CollisionRect(Aabb::new(center, size));
    }
}

fn update_speed(game: Res<Game>, mut query: Query<(&SpeedStep, &mut Speed)>) {
    for (speed_step, mut speed) in query.iter_mut() {
        speed.0 += speed_step.0;
        if speed.0 > game.ball_max_speed {
            speed.0 = game.ball_max_speed;
        }
    }
}

fn check_score(
    mut commands: Commands,
    mut game: ResMut<Game>,
    score_text_query: Query<Entity, With<ScoreText>>,
    ball_query: Query<(Entity, &Transform), With<Ball>>,
) {
    let game_area = game.area;
    for (entity, transform) in ball_query.iter() {
        let ball_x = transform.translation.x;
        let ball_out_left = ball_x < game_area.left;
        let ball_out_right = ball_x > game_area.right;
        let ball_out = ball_out_left || ball_out_right;
        if !ball_out {
            continue;
        }

        if ball_out_left {
            game.score.1 += 1;
        } else if ball_out_right {
            game.score.0 += 1;
        }

        commands.entity(entity).insert(NeedsReset);

        for entity in score_text_query.iter() {
            commands.entity(entity).insert(Dirty);
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Game {
            rng: rand::make_rng(),
            area: Aabb::new(Vec2::ZERO, Vec2::new(800.0, 600.0)),
            score: (0, 0),
            default_ball_speed: 100.0,
            ball_max_speed: 1000.0,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "PONG".to_string(),
                resolution: WindowResolution::new(800, 600),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                paddle_input,
                close_on_esc,
                update_score_text,
                reset_ball,
                wait_ball_launch_timer,
                launch_ball,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                move_paddle,
                move_ball,
                clamp_position_to_game_area_top_and_bottom,
                bounce_ball_on_paddle,
                update_collision_rect,
                update_speed,
                check_score,
            )
                .chain(),
        )
        .run();
}
