// https://www.youtube.com/watch?v=_C28kqin94c

use bevy::{prelude::*, window::PrimaryWindow};
use rand::{rngs::ThreadRng, thread_rng, Rng};

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Flappy Bird".to_string(),
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    resolution: Vec2::new(512.0, 512.0).into(),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(ImagePlugin::default_nearest()),
    );
    app.add_systems(Startup, setup_level);
    app.add_systems(Update, (update_bird, update_obstacles));
    app.run();
}

// bird
const PIXEL_RATIO: f32 = 4.0;
const FLAP_FORCE: f32 = 500.0;
const GRAVITY: f32 = 2000.0;
const VELOCITY_TO_ROTATION_RATIO: f32 = 7.5;

// obstacles
const OBSTACLE_AMOUNT: i32 = 5;
const OBSTACLE_WIDTH: f32 = 32.0;
const OBSTACLE_HEIGHT: f32 = 144.0;
const OBSTACLE_VERTICLE_OFFSET: f32 = 30.0;
const OBSTACLE_GAP_SIZE: f32 = 15.0;
const OBSTACLE_SPACING: f32 = 60.0;
const OBSTACLE_SCROLL_SPEED: f32 = 150.0;

// TODO: dont need to be public?

#[derive(Resource)]
struct GameManager {
    pipe_image: Handle<Image>,
    window_dimensions: Vec2,
}

#[derive(Component)]
struct Bird {
    pub y_velocity: f32,
}

#[derive(Component)]
struct Obstacle {
    pipe_direction: f32,
}

fn update_obstacles(
    time: Res<Time>,
    game_manager: Res<GameManager>,
    mut obstacle_query: Query<(&mut Obstacle, &mut Transform)>,
) {
    let mut rand = thread_rng();
    let y_offset = generate_offset(&mut rand);
    for (obstacle, mut transform) in obstacle_query.iter_mut() {
        transform.translation.x -= time.delta_secs() * OBSTACLE_SCROLL_SPEED;

        if transform.translation.x + OBSTACLE_WIDTH * PIXEL_RATIO / 2.0
            < -game_manager.window_dimensions.x / 2.0
        {
            transform.translation.x += OBSTACLE_AMOUNT as f32 * OBSTACLE_SPACING * PIXEL_RATIO;
            transform.translation.y =
                get_centered_pipe_position() + obstacle.pipe_direction * y_offset;
        }
    }
}

fn setup_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let pipe_image = asset_server.load("pipe.png");
    let window = window_query.get_single().expect("No window found");

    // insert resources
    commands.insert_resource(GameManager {
        pipe_image: pipe_image.clone(),
        window_dimensions: Vec2::new(window.width(), window.height()),
    });

    commands.insert_resource(ClearColor(Color::srgb(0.5, 0.7, 0.8)));
    commands.spawn(Camera2d::default());
    commands.spawn((
        Sprite {
            image: asset_server.load("bird.png"),
            ..Default::default()
        },
        Transform::IDENTITY.with_scale(Vec3::splat(PIXEL_RATIO)),
        Bird { y_velocity: 0.0 },
    ));

    let mut rand = thread_rng();

    spawn_obstacles(&mut commands, &mut rand, window.width(), &pipe_image);
}

fn update_bird(
    mut commands: Commands,
    mut bird_query: Query<(&mut Bird, &mut Transform), Without<Obstacle>>,
    mut obstacle_query: Query<(&Transform, Entity), With<Obstacle>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    game_manager: Res<GameManager>,
) {
    // only one bird in the game
    if let Ok((mut bird, mut transform)) = bird_query.get_single_mut() {
        if keys.just_pressed(KeyCode::Space) {
            bird.y_velocity = FLAP_FORCE;
        }

        bird.y_velocity -= time.delta_secs() * GRAVITY;
        transform.translation.y += bird.y_velocity * time.delta_secs();
        transform.rotation = Quat::from_axis_angle(
            Vec3::Z,
            f32::clamp(bird.y_velocity / VELOCITY_TO_ROTATION_RATIO, -90.0, 90.0).to_radians(),
        );

        let mut dead = false;

        // dropped to bottom
        if transform.translation.y <= -game_manager.window_dimensions.y / 2.0 {
            dead = true;
        }
        // see if collide
        else {
            for (pipe_transform, _entity) in obstacle_query.iter() {
                if (pipe_transform.translation.y - transform.translation.y).abs()
                    < OBSTACLE_HEIGHT * PIXEL_RATIO / 2.0
                    && (pipe_transform.translation.x - transform.translation.x).abs()
                        < OBSTACLE_WIDTH * PIXEL_RATIO / 2.0
                {
                    dead = true;
                    break;
                }
            }
        }

        if dead {
            transform.translation = Vec3::ZERO;
            bird.y_velocity = 0.0;
            for (_pipe_transform, entity) in obstacle_query.iter_mut() {
                commands.entity(entity).despawn();
            }
            let mut rand = thread_rng();
            spawn_obstacles(
                &mut commands,
                &mut rand,
                game_manager.window_dimensions.x,
                &game_manager.pipe_image,
            );
        }
    }
}

fn get_centered_pipe_position() -> f32 {
    return (OBSTACLE_HEIGHT / 2.0 + OBSTACLE_GAP_SIZE) * PIXEL_RATIO;
}

fn spawn_obstacles(
    commands: &mut Commands,
    rand: &mut ThreadRng,
    window_width: f32,
    pipe_image: &Handle<Image>,
) {
    for i in 0..OBSTACLE_AMOUNT {
        let y_offset = generate_offset(rand);
        let x_pos = window_width / 2.0 + (OBSTACLE_SPACING * PIXEL_RATIO * i as f32);
        spawn_obstacle(
            Vec3::new(x_pos, get_centered_pipe_position() + y_offset, 0.0),
            1.0,
            commands,
            pipe_image,
        );
        spawn_obstacle(
            Vec3::new(x_pos, -get_centered_pipe_position() + y_offset, 0.0),
            -1.0,
            commands,
            pipe_image,
        );
    }
}

fn spawn_obstacle(
    translation: Vec3,
    pipe_direction: f32,
    commands: &mut Commands,
    pipe_image: &Handle<Image>,
) {
    commands.spawn((
        Sprite {
            image: pipe_image.clone(),
            ..Default::default()
        },
        Transform::from_translation(translation).with_scale(Vec3::new(
            PIXEL_RATIO,
            PIXEL_RATIO * -pipe_direction,
            PIXEL_RATIO,
        )),
        Obstacle { pipe_direction },
    ));
}

fn generate_offset(rand: &mut ThreadRng) -> f32 {
    return rand.gen_range(-OBSTACLE_VERTICLE_OFFSET..OBSTACLE_VERTICLE_OFFSET) * PIXEL_RATIO;
}
