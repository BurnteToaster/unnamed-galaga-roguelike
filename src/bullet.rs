extern crate bevy;
use bevy::prelude::*;
use crate::LevelInfo;

#[derive(Component)]
pub struct Bullet {
    pub direction: Vec3,
}

const BULLET_SPEED: f32 = 500.0;

pub fn move_bullets(
    time: Res<Time>, 
    mut q_bullets: Query<(&mut Bullet, &mut Transform), With<Bullet>>,
    level_info: Res<LevelInfo>,
) {
    for (bullet, mut transform) in q_bullets.iter_mut() {
        transform.translation += BULLET_SPEED * time.delta_seconds() * level_info.time_scale * bullet.direction;
    }
}

pub fn despawn_projectile(
    windows: Query<&Window>,
    mut q_bullets: Query<(Entity, &Transform), With<Bullet>>,
    mut commands: Commands,
) {
    let window = windows.single();
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;

    for (entity, transform) in q_bullets.iter_mut() {
        if transform.translation.x < -half_width 
            || transform.translation.x > half_width 
            || transform.translation.y < -half_height 
            || transform.translation.y > half_height 
            {
            commands.entity(entity).despawn();
        }
    }
}