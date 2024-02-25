use bevy::render::camera::Camera;
use bevy::window::Window;
use bevy::prelude::*;
use bevy::ecs::event::ManualEventReader;
use rand::Rng;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuSettings, Backends};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends:Some(Backends::DX12),
                ..default()
                   })
        }))
        .add_systems(Startup, setup_game)
        .add_systems(Update, move_player)
        .add_systems(Update, spawn_projectile)
        .add_systems(Update, move_bullets)
        .add_systems(Update, update_cursor_position)
        .add_systems(Update, update_level_info)
        .add_systems(Update, spawn_enemies)
        .add_systems(Update, move_enemies)
        //.add_systems(Update, enemy_collision)
        .run();
}

#[derive(Component)]
struct Player {
    jump_cooldown: Timer,
    shot_cooldown: Timer,
    shot_limit: u32,
    position: Vec2,
}

#[derive(Component)]
struct Enemy {
    direction: Vec3,
    health: u32,
}

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Resource)]
struct LastCursorPosition(Vec2);

#[derive(Resource)]
struct LevelNumber(u32);

#[derive(Resource)]
struct EnemiesLeft(u32);

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

const PADDLE_SIZE: Vec3 = Vec3::new(40.0, 40.0, 0.0);
const PADDLE_COLOR: Color = Color::rgb(0.0, 0.0, 0.0);
const PLAYER_SPEED: f32 = 100.0;
const BULLET_SIZE: Vec3 = Vec3::new(10.0, 25.0, 0.0);
const BULLET_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);
const BULLET_SPEED: f32 = 500.0;
const ENEMY_SIZE: Vec3 = Vec3::new(35.0,35.0, 0.0);
const ENEMY_COLOR: Color = Color::rgb(1.0, 0.0, 0.75);
const ENEMY_SPEED: f32 = 75.0;

fn setup_game(
    mut commands: Commands,
    ) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -300.0, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Player {
            jump_cooldown: Timer::from_seconds(0.66, TimerMode::Once),
            shot_cooldown: Timer::from_seconds(0.5, TimerMode::Once),
            shot_limit: 3,
            position: Vec2::new(1080.0/2.0, (920.0/2.0)-300.0),
        },
        Collider,
    ));

    commands.insert_resource(LastCursorPosition(Vec2::new(0.0, 0.0)));
    commands.insert_resource(LevelNumber(1));
    commands.insert_resource(EnemiesLeft(2));
    commands.insert_resource(EnemySpawnTimer(Timer::from_seconds(5.0, TimerMode::Repeating)));
}

fn move_player(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Player), With<Player>>,
) {
    for (mut transform, mut player) in query.iter_mut() {
        let mut direction = -0.5;
        transform.translation.x += direction * PLAYER_SPEED * time.delta_seconds();
        if player.jump_cooldown.finished() {
            if keyboard_input.pressed(KeyCode::Key1) && !keyboard_input.just_released(KeyCode::Key1)
            {
                direction = 30.0;
                transform.translation.x += direction * PLAYER_SPEED * time.delta_seconds();
                player.jump_cooldown.reset();
            }
            if keyboard_input.pressed(KeyCode::Key2) && !keyboard_input.just_released(KeyCode::Key2)
            {
                direction = 60.0;
                transform.translation.x += direction * PLAYER_SPEED * time.delta_seconds();
                player.jump_cooldown.reset();
            }
            if keyboard_input.pressed(KeyCode::Key3) && !keyboard_input.just_released(KeyCode::Key3)
            {
                direction = 90.0;
                transform.translation.x += direction * PLAYER_SPEED * time.delta_seconds();
                player.jump_cooldown.reset();
            }
        }
        player.position = Vec2::new(transform.translation.x, transform.translation.y);
        player.jump_cooldown.tick(time.delta());
    }
}

fn update_cursor_position(
    cursor_events: Res<Events<CursorMoved>>,
    mut cursor_moved_reader: Local<ManualEventReader<CursorMoved>>,
    mut last_cursor_position: ResMut<LastCursorPosition>,
    query: Query<(&Camera, &GlobalTransform)>,
) {
    for event in cursor_moved_reader.read(&cursor_events) {
        let (camera, transform) = query.single();
        let world_position = camera.viewport_to_world(transform, event.position);
        last_cursor_position.0 = world_position.unwrap().origin.truncate();
    }
}

fn spawn_projectile(
    last_cursor_position: Res<LastCursorPosition>,
    time: Res<Time>,
    key_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Player), With<Player>>,
) {
    let cursor_position = last_cursor_position.0;
    let direction: Vec2 = cursor_position - query.single_mut().1.position;
    let direction = direction.normalize().extend(0.0);
    for (transform, mut player) in query.iter_mut() {
        if player.shot_cooldown.finished() && player.shot_limit != 0 {
            if key_input.pressed(MouseButton::Left) && !key_input.just_released(MouseButton::Left){
                let player_top = transform.translation.y + PADDLE_SIZE.y / 2.0;
                commands.spawn((
                    SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(transform.translation.x, player_top, 0.0),
                            scale: BULLET_SIZE,
                            ..default()
                        },
                        sprite: Sprite {
                            color: BULLET_COLOR,
                            ..default()
                        },
                        ..default()
                    },
                    Bullet,
                    Velocity(direction),
                    ),
                );
                player.shot_limit -= 1;
            }
            player.shot_cooldown.reset();
        }
        player.shot_cooldown.tick(time.delta());
    }
}

fn move_bullets(
    windows: Query<&Window>,
    time: Res<Time>, 
    mut query: Query<(Entity, &mut Transform, &mut Velocity), With<Bullet>>,
    mut q_player: Query<&mut Player>,
    mut commands: Commands,
) {
    let window = windows.single();
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;

    for (entity, mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * BULLET_SPEED * time.delta_seconds(); 
        if transform.translation.x < -half_width 
            || transform.translation.x > half_width 
            || transform.translation.y < -half_height 
            || transform.translation.y > half_height {
            q_player.single_mut().shot_limit += 1;
            commands.entity(entity).despawn();
        }
    }
}

fn update_level_info(
    mut level_number: ResMut<LevelNumber>,
    mut enemies_left: ResMut<EnemiesLeft>,
) {
    if enemies_left.0 == 0 {
        let prev_enemies_left = enemies_left.0;
        level_number.0 += 1;
        enemies_left.0 = (prev_enemies_left * 2) + 1;
        println!("Level: {}", level_number.0);
        println!("Enemies: {}", enemies_left.0);
    }
}

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    enemies_left: Res<EnemiesLeft>,
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    windows: Query<&Window>,
    q_player: Query<&Player>,
) {
    let window = windows.single();
    let buffer = 50.0; // Adjust this value as needed
    let spawn_width = window.width() - buffer;
    for _ in 0..enemies_left.0 {
        if enemy_spawn_timer.0.finished() {
            let mut rng = rand::thread_rng();
            let initial_x = rng.gen_range(-spawn_width / 2.0..spawn_width / 2.0);
            let player = q_player.single();
            let player_pos = player.position.extend(0.0);
            let direction = (player_pos - Vec3::new(initial_x, 500.0, 0.0)).normalize();
            commands.spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(initial_x, 500.0, 0.0),
                        scale: ENEMY_SIZE,
                        ..default()
                    },
                    sprite: Sprite {
                        color: ENEMY_COLOR,
                        ..default()
                    },
                    ..default()
                },
                Enemy{direction: direction, health: 1},
                Collider,
            ));
            enemy_spawn_timer.0.reset();
        } 
        else {
            enemy_spawn_timer.0.tick(time.delta());
        }
    } 
}

fn move_enemies(
    time: Res<Time>,
    mut q_enemies: Query<(&mut Transform, &mut Enemy), With<Enemy>>,
) {
    for (mut transform, enemy) in q_enemies.iter_mut() {
        let displacement;
        if transform.translation.y < -175.0 {
            displacement = Vec3::new(0.0, -1.0, 0.0);
        } else {
            let direction = enemy.direction;
            let x_axis = (direction.x, direction.y);
            let y_axis = (-direction.y, direction.x);

            let sine_wave = (time.elapsed_seconds() * 2.0 * std::f32::consts::PI).sin();
            displacement = Vec3::new(
                x_axis.0 + sine_wave * y_axis.0,
                x_axis.1 + sine_wave * y_axis.1,
                0.0,
            );
        }
        transform.translation += displacement * ENEMY_SPEED * time.delta_seconds();
    }
}

//THEN MAKE THE ENEMIES COLLIDE WITH THE PLAYER
/*fn enemy_collision(
    mut commands: Commands,
    mut q_enemies: Query<(&mut Transform, &mut Enemy), With<Enemy>>,
    q_player: Query<&Player, Entity>,
) {
    let player = q_player.single();
    let player_pos = player.position.extend(0.0);

    for (mut transform, _enemy) in q_enemies.iter_mut() {
        let enemy_pos = transform.translation;
        let distance = enemy_pos.distance(player_pos);
        if distance < 50.0 {
            println!("Game Over");
            // Despawn player
            if let Ok(entity) = q_player.entity() {
                commands.despawn(entity);
            }
        }
    }
}*/
//THEN MAKE THE ENEMIES COLLIDE WITH THE BULLETS
//THEN MAKE THE PLAYER COLLIDE WITH THE BULLETS
//THEN ADD A SCORE SYSTEM