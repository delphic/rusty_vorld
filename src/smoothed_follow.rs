use bevy::prelude::*;
use std::ops::Mul;

#[derive(Component)]
pub struct SmoothedFollow {
    pub target: Entity,
    pub translation_rate: f32,
    pub translation_offset: Vec3,
    pub rotation_rate: f32,
}

pub fn follow(
    mut follower_query: Query<(&SmoothedFollow, &mut Transform)>,
    transform_query: Query<&Transform, Without<SmoothedFollow>>,
) {
    for (smoothed_follow, mut transform) in follower_query.iter_mut() {
        if let Ok(target_transform) = transform_query.get(smoothed_follow.target) {
            let from_pos = transform.translation - transform.rotation.mul(smoothed_follow.translation_offset);
            transform.rotation = transform.rotation.slerp(target_transform.rotation, smoothed_follow.rotation_rate);
            // Arguably could use a smooth damp but lerp does fine
            transform.translation = from_pos.lerp(
                    target_transform.translation, smoothed_follow.translation_rate
                ) + transform.rotation.mul(smoothed_follow.translation_offset);
        }
    }
}