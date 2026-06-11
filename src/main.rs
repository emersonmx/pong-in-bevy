use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    window::WindowResolution,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use rand::{RngExt, rngs::ChaCha8Rng};

const PADDLE_SIZE: Vec2 = Vec2::new(20.0, 100.0);
const BALL_SIZE: Vec2 = Vec2::new(10.0, 10.0);

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

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource)]
struct Game {
    area: Rectangle,
    score: (u32, u32),
    default_paddle_speed: f32,
    default_ball_speed: f32,
    ball_speed_step: f32,
    ball_max_speed: f32,
}

#[derive(Debug, Resource, Deref, DerefMut)]
struct GameRng(ChaCha8Rng);

fn setup(game: Res<Game>, mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Name::new("MiddleLine"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Sprite::from_color(Color::WHITE, Vec2::new(1.0, game.area.size().y)),
    ));

    let wall_offset = game.area.half_size.y;
    commands.spawn((
        Name::new("TopWall"),
        Wall,
        CollisionPlane(Plane2d::new(Vec2::NEG_Y)),
        Transform::from_xyz(0.0, wall_offset, 0.0),
    ));
    commands.spawn((
        Name::new("BottomWall"),
        Wall,
        CollisionPlane(Plane2d::new(Vec2::Y)),
        Transform::from_xyz(0.0, -wall_offset, 0.0),
    ));

    let respawn_offset = game.area.half_size.x;
    commands.spawn((
        Name::new("LeftRespawn"),
        RespawnBallArea,
        Transform::from_xyz(-respawn_offset, 0.0, 0.0),
    ));
    commands.spawn((
        Name::new("RightRespawn"),
        RespawnBallArea,
        Transform::from_xyz(respawn_offset, 0.0, 0.0),
    ));

    let paddle_offset = game.area.half_size.x - 20.0;
    commands.spawn((
        Name::new("LeftPaddle"),
        Paddle,
        PaddleKeyboardInput {
            up_key: KeyCode::KeyW,
            down_key: KeyCode::KeyS,
        },
        Speed(game.default_paddle_speed),
        Direction::default(),
        CollisionRect(Rectangle::from_size(PADDLE_SIZE)),
        Transform::from_xyz(-paddle_offset, 0.0, 0.0),
        Sprite::from_color(Color::WHITE, PADDLE_SIZE),
    ));

    commands.spawn((
        Name::new("RightPaddle"),
        Paddle,
        Bot,
        Speed(game.default_paddle_speed),
        Direction::default(),
        CollisionRect(Rectangle::from_size(PADDLE_SIZE)),
        Transform::from_xyz(paddle_offset, 0.0, 0.0),
        Sprite::from_color(Color::WHITE, PADDLE_SIZE),
    ));

    commands.spawn((
        Name::new("Ball"),
        Ball,
        Speed(game.default_ball_speed),
        NeedsRespawn,
        CollisionRect(Rectangle::from_size(BALL_SIZE)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Sprite::from_color(Color::WHITE, BALL_SIZE),
    ));

    commands
        .spawn((
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
        ))
        .with_children(|parent| {
            parent.spawn((Name::new("Score"), Text::default(), ScoreText, Dirty));
        });
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
    mut scores: Query<(Entity, &mut Text), (With<ScoreText>, With<Dirty>)>,
) {
    let score = format!("{}   {}", game.score.0, game.score.1);
    for (entity, mut text) in &mut scores {
        *text = Text::new(&score);
        commands.entity(entity).remove::<Dirty>();
    }
}

fn respawn_ball(
    mut commands: Commands,
    mut balls: Query<(Entity, &mut Transform), (With<Ball>, With<NeedsRespawn>)>,
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
    mut commands: Commands,
    mut balls: Query<(Entity, &mut LaunchTimer), With<Ball>>,
) {
    for (entity, mut timer) in &mut balls {
        if timer.tick(time.delta()).just_finished() {
            commands.entity(entity).insert(NeedsLaunch);
            commands.entity(entity).remove::<LaunchTimer>();
        }
    }
}

fn launch_ball(
    game: Res<Game>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
    mut balls: Query<(Entity, &mut Speed), (With<Ball>, With<NeedsLaunch>)>,
) {
    for (entity, mut speed) in &mut balls {
        let dir_x = if rng.random() { 1.0 } else { -1.0 };
        let dir_y = rng.random::<f32>() * 2.0 - 1.0;
        let new_dir = Vec3::new(dir_x, dir_y, 0.0).normalize();

        commands.entity(entity).insert(Velocity(new_dir));
        speed.0 = game.default_ball_speed;
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
    mut commands: Commands,
    mut game: ResMut<Game>,
    score_text_query: Query<Entity, With<ScoreText>>,
    respawns: Query<&Transform, With<RespawnBallArea>>,
    balls: Query<(Entity, &Transform), With<Ball>>,
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
                game.score.1 += 1;
            } else if ball_out_right {
                game.score.0 += 1;
            }

            commands.entity(entity).insert(NeedsRespawn);

            for entity in &score_text_query {
                commands.entity(entity).insert(Dirty);
            }
        }
    }
}

fn update_speed(game: Res<Game>, mut speeds: Query<&mut Speed>) {
    for mut speed in &mut speeds {
        speed.0 += game.ball_speed_step;
        if speed.0 > game.ball_max_speed {
            speed.0 = game.ball_max_speed;
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GameRng(rand::make_rng()))
        .insert_resource(Game {
            area: Rectangle::new(800.0, 600.0),
            score: (0, 0),
            default_paddle_speed: 300.0,
            default_ball_speed: 100.0,
            ball_speed_step: 0.1,
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
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, (paddle_input, bot_input, close_on_esc))
        .add_systems(Update, update_score_text)
        .add_systems(
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
                .chain(),
        )
        .run();
}
