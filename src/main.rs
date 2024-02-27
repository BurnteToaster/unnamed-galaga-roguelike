mod player;
mod enemy;
mod bullet;

use player::{Player, move_player, spawn_projectile};
use enemy::{spawn_enemies, move_enemies, enemy_collision_bullet};
use bullet::{move_bullets, despawn_projectile};
use bevy::render::camera::Camera;
use bevy::prelude::*;
use bevy::ecs::event::ManualEventReader;
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
        .add_systems(Update, update_time_scale)
        .add_systems(Update, despawn_projectile)
        .add_systems(Update, enemy_collision_bullet)
        .insert_resource(LastCursorPosition(Vec2::new(0.0, 0.0)))
        .insert_resource(LevelNumber(1))
        .insert_resource(EnemiesLeft(2))
        .insert_resource(EnemySpawnTimer(Timer::from_seconds(5.0, TimerMode::Repeating)))
        .insert_resource(TimeScale(1.0))
        .run();
}

#[derive(Resource)]
struct LastCursorPosition(Vec2);

#[derive(Resource)]
struct LevelNumber(u32);

#[derive(Component)]
struct Collider; //move to player.rs after main menu

#[derive(Resource)]
struct EnemiesLeft(u32);

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

#[derive(Resource)]
struct TimeScale(f32);

const PADDLE_SIZE: Vec3 = Vec3::new(0.15, 0.20, 0.0);

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(( //move bundle to player.rs after main menu
        SpriteBundle {
            texture: asset_server.load("ship.png"),
            transform: Transform {
                translation: Vec3::new(0.0, -300.0, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            ..default()
        },
        Player {
            movement_speed: 1000.0,
            shot_cooldown: Timer::from_seconds(0.5, TimerMode::Once),
            shot_limit: 3,
            position: Vec2::new(1080.0/2.0, (920.0/2.0)-300.0),
        },
        Collider,
    ));
}

fn update_time_scale(input: Res<Input<MouseButton>>, mut time_scale: ResMut<TimeScale>) {
    if input.pressed(MouseButton::Right) {
        time_scale.0 = 0.5;
    } else {
        time_scale.0 = 1.0;
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

//THEN MAKE THE ENEMIES COLLIDE WITH THE BULLETS
//THEN MAKE THE PLAYER COLLIDE WITH THE BULLETS
//THEN ADD A SCORE SYSTEM