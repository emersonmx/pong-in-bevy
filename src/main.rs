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

#[derive(Debug, Component)]
enum Side {
    Left,
    Right,
}

#[derive(Debug, Default, Component, Deref, DerefMut)]
struct Direction(Vec2);

#[derive(Debug, Default, Component, Deref, DerefMut)]
struct Speed(f32);

#[derive(Debug, Default, Component, Deref, DerefMut)]
struct DefaultSpeed(f32);

#[derive(Debug, Default, Component, Deref, DerefMut)]
struct MaxSpeed(f32);

#[derive(Component)]
struct SpeedStep(f32);

#[derive(Debug, Default, Component, Deref, DerefMut)]
struct Velocity(Vec3);

#[derive(Debug, Component, Deref, DerefMut)]
struct CollisionRect(Aabb);

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct ScoreText;

#[derive(Debug, Resource)]
struct Game {
    rng: ChaCha8Rng,
    area: Aabb,
    score: (u32, u32),
}

fn setup(mut game: ResMut<Game>, mut commands: Commands) {
    let game_area = game.area;

    commands.spawn(Camera2d);

    commands.spawn((
        Transform::from_translation(game_area.center().extend(0.0)),
        Sprite::from_color(Color::WHITE, Vec2::new(1.0, 600.0)),
    ));

    let paddle_size = Vec2::new(20.0, 100.0);
    let left_position = Vec3::new(game_area.left + paddle_size.x, 0.0, 0.0);
    commands.spawn((
        Paddle,
        Side::Left,
        Speed(300.0),
        Direction::default(),
        CollisionRect(Aabb::new(left_position.truncate(), paddle_size)),
        Transform::from_translation(left_position),
        Sprite::from_color(Color::WHITE, paddle_size),
    ));

    let right_position = Vec3::new(game_area.right - paddle_size.x, 0.0, 0.0);
    commands.spawn((
        Paddle,
        Side::Right,
        Speed(300.0),
        Direction::default(),
        CollisionRect(Aabb::new(right_position.truncate(), paddle_size)),
        Transform::from_translation(right_position),
        Sprite::from_color(Color::WHITE, paddle_size),
    ));

    let ball_size = Vec2::new(10.0, 10.0);
    let ball_position = game_area.center();
    let ball_dir = Vec3::new(
        if game.rng.random() { 1.0 } else { -1.0 },
        game.rng.random(),
        0.0,
    )
    .normalize();
    let speed = 100.0;
    commands.spawn((
        Ball,
        Speed(speed),
        DefaultSpeed(speed),
        SpeedStep(0.1),
        MaxSpeed(1000.0),
        Velocity(ball_dir),
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
            parent.spawn((Text::default(), ScoreText));
        });
}

fn paddle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Side, &mut Direction), With<Paddle>>,
) {
    for (side, mut direction) in query.iter_mut() {
        let up_key = match side {
            Side::Left => KeyCode::KeyW,
            Side::Right => KeyCode::ArrowUp,
        };
        let down_key = match side {
            Side::Left => KeyCode::KeyS,
            Side::Right => KeyCode::ArrowDown,
        };

        direction.x = 0.0;
        direction.y = 0.0;

        if keyboard_input.pressed(up_key) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(down_key) {
            direction.y -= 1.0;
        }
    }
}

fn update_score_text(game: Res<Game>, mut query: Query<&mut Text, With<ScoreText>>) {
    let score = format!("{}   {}", game.score.0, game.score.1);
    for mut text in query.iter_mut() {
        *text = Text::new(&score);
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

fn update_speed(mut query: Query<(&MaxSpeed, &SpeedStep, &mut Speed)>) {
    for (max_speed, speed_step, mut speed) in query.iter_mut() {
        speed.0 += speed_step.0;
        if speed.0 > max_speed.0 {
            speed.0 = max_speed.0;
        }
    }
}

fn check_score_and_reset_ball(
    mut game: ResMut<Game>,
    mut ball_query: Query<(&DefaultSpeed, &mut Transform, &mut Velocity, &mut Speed), With<Ball>>,
) {
    let game_area = game.area;
    for (default_speed, mut transform, mut velocity, mut speed) in ball_query.iter_mut() {
        let ball_x = transform.translation.x;
        let ball_out_left = ball_x < game_area.left;
        let ball_out_right = ball_x > game_area.right;

        if ball_out_left || ball_out_right {
            if ball_out_left {
                game.score.1 += 1;
            } else if ball_out_right {
                game.score.0 += 1;
            }

            let center = game_area.center().extend(0.0);
            transform.translation = center;

            let dir_x = if game.rng.random() { 1.0 } else { -1.0 };
            let dir_y = game.rng.random::<f32>() * 2.0 - 1.0;
            let new_dir = Vec3::new(dir_x, dir_y, 0.0).normalize();
            velocity.0 = new_dir;

            speed.0 = default_speed.0;
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

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Game {
            rng: rand::make_rng(),
            area: Aabb::new(Vec2::ZERO, Vec2::new(800.0, 600.0)),
            score: (0, 0),
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
        .add_systems(Update, (paddle_input, close_on_esc, update_score_text))
        .add_systems(
            FixedUpdate,
            (
                move_paddle,
                move_ball,
                clamp_position_to_game_area_top_and_bottom,
                bounce_ball_on_paddle,
                update_collision_rect,
                update_speed,
                check_score_and_reset_ball,
            )
                .chain(),
        )
        .run();
}
