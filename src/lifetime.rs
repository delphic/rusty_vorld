use bevy::prelude::*;

#[derive(Component)]
pub struct Lifetime {
    pub time_remaining: f32,
}

pub fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut lifetime_query: Query<(Entity, &mut Lifetime)>
) {
    for (entity, mut lifetime) in lifetime_query.iter_mut() {
        lifetime.time_remaining -= time.delta_seconds();
        if lifetime.time_remaining < 0.0 {
            commands.entity(entity).despawn();
        }
    }
}