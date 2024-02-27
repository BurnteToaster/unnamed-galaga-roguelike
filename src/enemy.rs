use bevy::{prelude::*, sprite::collide_aabb::collide};
use rand::Rng;
use crate::{bullet::Bullet, player::{Player, BULLET_SIZE}, EnemiesLeft, EnemySpawnTimer, TimeScale};

#[derive(Component)]
pub struct Enemy {
    movement_speed: f32,
    direction: Vec3,
    health: i32,
}

#[derive(Component)]
pub struct Collider;

const ENEMY_SIZE: Vec3 = Vec3::new(35.0,35.0, 0.0);
const ENEMY_COLOR: Color = Color::rgb(1.0, 0.0, 0.75);

pub fn spawn_enemies(
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
                Enemy{movement_speed: 75.0, direction: direction, health: 1},
                Collider,
            ));
            enemy_spawn_timer.0.reset();
        } 
        else {
            enemy_spawn_timer.0.tick(time.delta());
        }
    } 
}

pub fn move_enemies(
    time: Res<Time>,
    mut q_enemies: Query<(&mut Transform, &mut Enemy), With<Enemy>>,
    time_scale: Res<TimeScale>
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
        transform.translation += displacement * enemy.movement_speed * time.delta_seconds() * time_scale.0;
    }
}

//THEN MAKE THE ENEMIES COLLIDE WITH THE BULLETS
pub fn enemy_collision_bullet(
    mut commands: Commands,
    q_bullets: Query<(Entity, &Transform), With<Bullet>>,
    q_collider: Query<(Entity, &Transform, Option<&Enemy>), With<Collider>>,
) {
    for (bullet_entity, bullet_transform) in &q_bullets {
        for (enemy_entity, enemy_transform, enemy) in &q_collider {
            let collision = collide(
                bullet_transform.translation,
                BULLET_SIZE.truncate(),
                enemy_transform.translation,
                ENEMY_SIZE.truncate(),
            );
            if let Some(_) = collision {
                if enemy.is_some() {
                    commands.entity(bullet_entity).despawn();
                    commands.entity(enemy_entity).despawn();
                }
            }
        }
    }
}