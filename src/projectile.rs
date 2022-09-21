use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Copy, Clone)]
pub struct Projectile {
    pub damage: u32,
}

pub struct ProjectileImpactEvent {
    pub projectile: Projectile,
    pub hit_entity: Entity,
}

pub fn detect_projectile_impact(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    projectile_query: Query<&Projectile>,
    mut projectile_event_writer: EventWriter<ProjectileImpactEvent>,
) {
    for collision in collision_events.iter() {
        match collision {
            CollisionEvent::Started(entity1, entity2, _event_flags) => {
                if let Ok(projectile) = projectile_query.get(*entity1) {
                    projectile_event_writer.send(ProjectileImpactEvent { 
                        projectile: *projectile,
                        hit_entity: *entity2,
                    });
                    commands.entity(*entity1).despawn();
                } else if let Ok(projectile) = projectile_query.get(*entity2) {
                    projectile_event_writer.send(ProjectileImpactEvent {
                        projectile: *projectile,
                        hit_entity: *entity1,
                    });
                    commands.entity(*entity2).despawn();
                }
            },
            _ => { }
        }
    }
}