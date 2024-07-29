extern crate rand;
extern crate bevy;

mod player;
mod enemy;
mod bullet;

use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use player::{Player, move_player, spawn_projectile};
use enemy::{bullet_enemy_collision_system, move_enemies, Enemy};
use bullet::{move_bullets, despawn_projectile};

use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::ecs::event::ManualEventReader;
use rand::Rng;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    Menu,
    #[default]
    Game,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends:Some(Backends::DX12),
                ..default()
                   }),
            synchronous_pipeline_compilation: false,
        }))
        .init_state::<GameState>()
        .add_systems(Startup, setup_game)
        .add_systems(Update, display_ui)
        .add_systems(Update, move_player)
        .add_systems(Update, spawn_projectile)
        .add_systems(Update, move_bullets)
        .add_systems(Update, update_cursor_position)
        .add_systems(Update, update_level_info)
        .add_systems(Update, spawn_enemies)
        .add_systems(Update, move_enemies)
        .add_systems(Update, update_time_scale)
        .add_systems(Update, despawn_projectile)
        .add_systems(Update, bullet_enemy_collision_system)
        //.add_systems(Update, update_ui)
        .run();
}

#[derive(Resource)]
struct LastCursorPosition(Vec2);

#[derive(Component)]
struct Collider; //move to player.rs after main menu

#[derive(Resource)]
pub struct EnemiesLeft {
    prev: u32,
    curr: u32,
    next: u32,
}

#[derive(Resource)]
pub struct LevelInfo {
    level_number: u32,
    total_enemies: u32,
    enemy_spawn_timer: Timer,
    enemy_health: u32,
    time_scale: f32,
    level_transition_timer: Timer,
}

const PADDLE_SIZE: Vec3 = Vec3::new(0.15, 0.20, 0.0);
const ENEMY_SIZE: Vec3 = Vec3::new(35.0,35.0, 0.0);
const ENEMY_COLOR: Color = Color::rgb(1.0, 0.0, 0.75);

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

    commands.insert_resource(LastCursorPosition(Vec2::new(0.0, -200.0)));
    commands.insert_resource(EnemiesLeft{prev: 1, curr: 1, next: 2});
    commands.insert_resource(
        LevelInfo{
            level_number: 1, 
            total_enemies: 1, 
            enemy_spawn_timer: Timer::from_seconds(1.0, TimerMode::Once),
            enemy_health: 1,
            time_scale: 1.0,
            level_transition_timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    );
}

pub fn display_ui(
    mut commands: Commands,
    level_info: Res<LevelInfo>,
) {
    
    let mut level = format!("Level: {}\n", level_info.level_number);
    let mut enemies = format!("Enemies: {}\n", level_info.total_enemies);
    let mut timescale = format!("Timescale: {}\n", level_info.time_scale);
    commands.spawn(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: level,
                    style: TextStyle {
                        font_size: 30.0,
                        color: Color::WHITE,
                        ..default()
                    },
                },
                TextSection {
                    value: enemies,
                    style: TextStyle {
                        font_size: 30.0,
                        color: Color::WHITE,
                        ..default()
                    },
                },
                TextSection {
                    value: timescale,
                    style: TextStyle {
                        font_size: 30.0,
                        color: Color::WHITE,
                        ..default()
                    },
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    });
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    windows: Query<&Window>,
    q_player: Query<&Player>,
    mut level_info: ResMut<LevelInfo>,
) {
    let window = windows.single();
    let buffer = 50.0; // Adjust this value as needed
    let spawn_width = window.width() - buffer;
    if level_info.enemy_spawn_timer.finished() && level_info.level_transition_timer.finished() {
        let mut rng = rand::thread_rng();
        let initial_x = rng.gen_range(-spawn_width / 2.0..spawn_width / 2.0);
        let player = q_player.single();
        let above_player_pos = player.position.extend(0.0) + Vec3::new(0.0, -175.0, 0.0);
        let direction = (above_player_pos - Vec3::new(initial_x, 500.0, 0.0)).normalize();
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
            Enemy{movement_speed: 75.0, direction: direction, health: level_info.enemy_health},
            Collider,
        ));
        level_info.enemy_spawn_timer.reset();
    } 
    else {
        level_info.enemy_spawn_timer.tick(time.delta());
    }
}

fn update_time_scale(input: Res<ButtonInput<MouseButton>>, mut level_info: ResMut<LevelInfo> ) {
    if input.pressed(MouseButton::Right) {
        level_info.time_scale = 0.5;
    } else {
        level_info.time_scale = 1.0;
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
    mut commands: Commands,
    time: Res<Time>,
    mut level_info: ResMut<LevelInfo>,
    mut enemies_left: ResMut<EnemiesLeft>,
    q_enemies: Query<(&mut Enemy, Entity)>,
) {
    if enemies_left.curr == 0 {
        level_info.enemy_spawn_timer.pause();
        if level_info.level_transition_timer.finished() {
            level_info.level_number += 1;
            enemies_left.prev = level_info.total_enemies;
            if level_info.level_number % 5 == 0 {
                level_info.enemy_health += 1;
                enemies_left.next -= 5;
            }
            enemies_left.curr = enemies_left.next;
            enemies_left.next = enemies_left.curr + enemies_left.prev;
            level_info.total_enemies = enemies_left.curr;

            println!("Level: {}", level_info.level_number);
            println!("Enemies: {}", level_info.total_enemies);
            level_info.enemy_spawn_timer.reset();
            level_info.enemy_spawn_timer.unpause();
        }
    }
    level_info.level_transition_timer.tick(time.delta());
}

/*fn update_ui(
    mut q_ui: Query<(&UI, &mut Text)>,
) {
    for (ui, mut text) in q_ui.iter_mut() {
        text.sections[0].value = format!("Level: {}\n", ui.Level);
        text.sections[1].value = format!("Enemies: {}\n", ui.Enemies);
        text.sections[2].value = format!("Timescale: {}\n", ui.TimeScale);
    }
}*/

//THEN MAKE THE PLAYER COLLIDE WITH THE BULLETS
//THEN ADD A SCORE SYSTEM