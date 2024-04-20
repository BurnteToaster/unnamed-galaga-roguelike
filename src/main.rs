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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends:Some(Backends::DX12),
                ..default()
                   }),
            synchronous_pipeline_compilation: false,
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
        .add_systems(Update, bullet_enemy_collision_system)
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum GameState {
    Menu,
    Playing,
}

#[derive(Resource)]
struct MenuComponents {
    container: Camera2dBundle,
    play_button: Entity,
    quit_button: Entity,
}


#[derive(Resource)]
struct LastCursorPosition(Vec2);

#[derive(Component)]
struct Collider; //move to player.rs after main menu

#[derive(Default, Resource)]
pub struct EnemiesLeft {
    prev: u32,
    curr: u32,
    next: u32,
}

#[derive(Default, Resource)]
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
            enemy_spawn_timer: Timer::from_seconds(1.5, TimerMode::Once),
            enemy_health: 1,
            time_scale: 1.0,
            level_transition_timer: Timer::from_seconds(5.0, TimerMode::Once),
        });

    commands.spawn(TextBundle {
        text: Text {
            sections: Vec::<TextSection>::new(),
            ..default()
        },
        background_color: BackgroundColor(Color::BLACK),
        ..default()       
    });

    let button_style = Style {
        margin: UiRect::new(Val::Px(0.0), Val::Px(10.0), Val::Px(0.0), Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..Default::default()
    };

    let play_button = commands
        .spawn(ButtonBundle {
            style: button_style.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Play",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    }
                ),
                ..Default::default()
            });
        })
        .id();

    let quit_button = commands
        .spawn(ButtonBundle {
            style: button_style.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Quit",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    }
                ),
                ..Default::default()
            });
        })
        .id();

    commands.insert_resource(MenuComponents {
        container: Camera2dBundle::default(),
        play_button: play_button,
        quit_button: quit_button,
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
    time: Res<Time>,
    mut level_info: ResMut<LevelInfo>,
    mut enemies_left: ResMut<EnemiesLeft>,
) {
    if enemies_left.curr == 0 {
        level_info.enemy_spawn_timer.pause();
        if level_info.level_transition_timer.finished() {
            level_info.level_number += 1;
            enemies_left.prev = enemies_left.curr;
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

//THEN MAKE THE PLAYER COLLIDE WITH THE BULLETS
//THEN ADD A SCORE SYSTEM