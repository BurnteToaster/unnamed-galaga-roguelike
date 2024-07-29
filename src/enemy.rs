extern crate bevy;
use bevy::{math::bounding::{Aabb2d, IntersectsVolume}, prelude::*};
use crate::{bullet::Bullet, player::BULLET_SIZE, ENEMY_SIZE, LevelInfo, EnemiesLeft, Collider};

#[derive(Component)]
pub struct Enemy {
    pub movement_speed: f32,
    pub direction: Vec3,
    pub health: u32,
}

pub fn move_enemies(
    time: Res<Time>,
    mut q_enemies: Query<(&mut Transform, &mut Enemy), With<Enemy>>,
    level_info: Res<LevelInfo>,
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
        transform.translation += displacement * enemy.movement_speed * time.delta_seconds() * level_info.time_scale;
    }
}

pub fn bullet_enemy_collision_system(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    mut enemy_query: Query<(&mut Enemy, Entity, &Transform), With<Collider>>,
    mut enemies_left: ResMut<EnemiesLeft>,
) {
    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        for (mut enemy, enemy_entity, enemy_transform) in enemy_query.iter_mut() {
            let collision = Aabb2d::new(
                bullet_transform.translation.xy(),
                BULLET_SIZE.xy()).intersects(
                &Aabb2d::new(
                    enemy_transform.translation.xy(),
                    ENEMY_SIZE.xy()
                )
            );
            
            if collision {
                commands.entity(bullet_entity).despawn(); // despawn the bullet
                enemy.health -= 1; // damage the enemy

                if enemy.health <= 0 {
                    commands.entity(enemy_entity).despawn(); // despawn the enemy
                    enemies_left.curr -= 1;
                }
            }
        }
    }
}