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

#[derive(Debug, Component, Deref, DerefMut, Default)]
struct Direction(Vec2);

#[derive(Debug, Component, Deref, DerefMut, Default)]
struct Speed(f32);

#[derive(Debug, Component, Deref, DerefMut, Default)]
struct Velocity(Vec3);

#[derive(Debug, Component, Deref, DerefMut)]
struct CollisionRect(Aabb);

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Debug, Resource)]
struct Game {
    rng: ChaCha8Rng,
    area: Aabb,
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
    commands.spawn((
        Ball,
        Speed(100.0),
        Velocity(ball_dir),
        CollisionRect(Aabb::new(ball_position, ball_size)),
        Transform::from_translation(ball_position.extend(0.0)),
        Sprite::from_color(Color::WHITE, ball_size),
    ));
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
                let ball_half_size = ball_collision_rect.half_size();
                transform.translation.x = if ball_velocity.x > 0.0 {
                    paddle_collision_rect.left - ball_half_size.x
                } else {
                    paddle_collision_rect.right + ball_half_size.x
                };

                let paddle_center = paddle_collision_rect.center();
                let ball_center = ball_collision_rect.center();
                let offset =
                    (ball_center.y - paddle_center.y) / paddle_collision_rect.half_size().y;
                let offset = offset.clamp(-1.0, 1.0);

                let new_direction = Vec2::new(-ball_velocity.x.signum(), offset).normalize();
                ball_velocity.x = new_direction.x * ball_velocity.length();
                ball_velocity.y = new_direction.y * ball_velocity.length();
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
        .add_systems(Update, (paddle_input, close_on_esc))
        .add_systems(
            FixedUpdate,
            (
                move_paddle,
                move_ball,
                clamp_position_to_game_area_top_and_bottom,
                bounce_ball_on_paddle,
                update_collision_rect,
            )
                .chain(),
        )
        .run();
}
