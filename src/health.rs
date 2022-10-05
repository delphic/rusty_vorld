use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::projectile;
use super::projectile::*;

#[derive(Component)]
pub struct Health {
    pub max_health: u32,
    pub current_health: u32,
}

pub struct TakeDamageEvent {
    pub entity: Entity, 
    pub damage_taken: u32,
    pub damage_inflicted: u32,
}

impl Health {
    pub fn new(health: u32) -> Self {
        Self {
            max_health: health,
            current_health: health,
        }
    }
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamageEvent>();
        app.add_system(handle_projectile_impact.after(projectile::detect_projectile_impact));
    }
}

fn handle_projectile_impact(
    mut commands: Commands,
    mut projectile_event_reader: EventReader<ProjectileImpactEvent>,
    mut take_damage_event_writer: EventWriter<TakeDamageEvent>,
    collider_parent_query: Query<Option<&Parent>, With<Collider>>,
    mut health_query: Query<(Entity, &mut Health)>,
) {
    for event in projectile_event_reader.iter() {
        if let Ok(parent_option) = collider_parent_query.get(event.hit_entity) {
            if let Ok((entity, health)) = health_query.get_mut(event.hit_entity) {
                inflict_damage(health, entity, &mut commands, event.projectile.damage, &mut take_damage_event_writer);
            } else  if let Some(parent) = parent_option {
                if let Ok((entity, health)) = health_query.get_mut(parent.get()) {
                    inflict_damage(health, entity, &mut commands, event.projectile.damage, &mut take_damage_event_writer);
                }
            }
        }
    }
}

fn inflict_damage(
    mut health: Mut<Health>, 
    hit_entity: Entity,
    commands: &mut Commands,
    damage: u32,
    take_damage_event_writer: &mut EventWriter<TakeDamageEvent>,
) {
    let previous_health = health.current_health;
    health.current_health -= damage.min(health.current_health);
    take_damage_event_writer.send(TakeDamageEvent {
        entity: hit_entity,
        damage_taken: previous_health - health.current_health,
        damage_inflicted: damage
    });
    if health.current_health <= 0 { // May want to handle this in response to TakeDamageEvent?
        commands.entity(hit_entity).despawn_recursive();
    }
}