use bevy::prelude::*;
use super::player::Player;

#[derive(Component)]
pub struct Zombie;

pub fn seek_brains(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Zombie>)>,
    mut zombie_query: Query<&mut Transform, With<Zombie>>
) {
    if let Some(player_transform) = player_query.iter().last() {
        for mut transform in zombie_query.iter_mut() {
            let target_position = Vec3::new(player_transform.translation.x, transform.translation.y, player_transform.translation.z);
            let target_direction = target_position - transform.translation;
            let distance_sqr = target_direction.length_squared();
            if distance_sqr < 8.0 * 8.0 && distance_sqr > 1.0 {
                transform.look_at(target_position, Vec3::Y);
                transform.translation += target_direction.normalize() * time.delta_seconds();
                // TODO: Play walk animation
            }
            // TODO: else play no animation
        }
    }
}