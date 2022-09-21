use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use super::projectile::*;

#[derive(Component)]
pub struct Health {
    pub max_health: u32,
    pub current_health: u32,
}

pub fn handle_projectile_impact(
    mut commands: Commands,
    mut projectile_event_reader: EventReader<ProjectileImpactEvent>,
    collider_parent_query: Query<&Parent, With<Collider>>,
    mut health_query: Query<(Entity, &mut Health)>,
) {
    for event in projectile_event_reader.iter() {
        if let Ok(parent) = collider_parent_query.get(event.hit_entity) {
            if let Ok((entity, mut health)) = health_query.get_mut(parent.get()) {
                health.current_health -= event.projectile.damage.min(health.current_health);
    
                if health.current_health <= 0 { // Ideally would emit DamageTaken event and respond with individual systems
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}