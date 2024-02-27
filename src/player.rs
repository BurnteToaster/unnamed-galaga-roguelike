use bevy::prelude::*;
use bevy::window::Window;
use bevy::input::mouse::MouseWheel;
use crate::{bullet::Bullet, TimeScale, LastCursorPosition};

#[derive(Component)]
pub struct Player {
    pub movement_speed: f32,
    pub shot_cooldown: Timer,
    pub shot_limit: u32,
    pub position: Vec2,
}

#[derive(Component)]
pub struct Collider;

#[derive(Component)]
pub struct Velocity(Vec3);

const PADDLE_SIZE: Vec3 = Vec3::new(0.15, 0.20, 0.0);
pub const BULLET_SIZE: Vec3 = Vec3::new(25.0, 10.0, 0.0);
const BULLET_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);

pub fn move_player(
    time: Res<Time>,
    mut wheel_input: EventReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut Player), With<Player>>,
    windows: Query<&Window>,
    time_scale: Res<TimeScale>
)
    {   
        let window = windows.single();
        let half_width = window.width() / 2.0;
        for (mut transform, mut player) in query.iter_mut() {
            for event in wheel_input.read() {
                let direction = event.y;
                if !(transform.translation.x < -half_width || transform.translation.x > half_width) {
                    transform.translation.x += direction * player.movement_speed * time.delta_seconds() * time_scale.0;
                }
                player.position = Vec2::new(transform.translation.x, transform.translation.y);
            }
        }
    }

    
pub fn spawn_projectile(
    last_cursor_position: Res<LastCursorPosition>,
    time: Res<Time>,
    key_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Player), With<Player>>,
) {
    let cursor_position = last_cursor_position.0;

    for (transform, mut player) in query.iter_mut() {
        if player.shot_cooldown.finished() && player.shot_limit != 0 {
            if key_input.pressed(MouseButton::Left) && !key_input.just_released(MouseButton::Left){
                let player_top = transform.translation.y + PADDLE_SIZE.y / 2.0;
                let player_center = Vec2::new(transform.translation.x, player_top);
                let direction: Vec2 = (cursor_position - player_center).normalize();
                let rotation = Quat::from_rotation_z(direction.y.atan2(direction.x));
                commands.spawn((
                    SpriteBundle {
                        transform: Transform {
                            translation: player_center.extend(0.0),
                            scale: BULLET_SIZE,
                            rotation: rotation,
                            ..default()
                        },
                        sprite: Sprite {
                            color: BULLET_COLOR,
                            ..default()
                        },
                        ..default()
                    },
                    Bullet{direction: direction.extend(0.0)},
                    Collider,
                    ),
                );
                player.shot_limit -= 1;
            }
            player.shot_cooldown.reset();
        }
        player.shot_cooldown.tick(time.delta());
        player.shot_limit += 1;
    }
}