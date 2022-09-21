use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::health::Health;
use super::input::PlayerInput;
use super::lifetime::*;
use super::named_collision_groups::*;
use super::player::PlayerCamera;
use super::zombie::Zombie;

pub struct BulletMeshMaterial {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>
}

#[derive(Component)]
pub struct Projectile {
    damage: u32,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::UVSphere { radius: 0.01, sectors: 8, stacks: 8 }));
    let material = materials.add(StandardMaterial { 
        base_color: Color::rgb_u8(50, 5, 5),
        perceptual_roughness: 0.2,
        ..default()
    });
    commands.insert_resource(BulletMeshMaterial { mesh, material });
}

pub fn shoot(
    mut commands: Commands,
    bullet_assets: Res<BulletMeshMaterial>,
    mut player_input: ResMut<PlayerInput>,
    transform_query: Query<&GlobalTransform, With<PlayerCamera>>, // TODO: Gun barrel component
) {
    
    if player_input.shoot_requested {
        player_input.shoot_requested = false;
        if let Some(global_transform) = transform_query.iter().last() {
            commands.spawn()
                .insert_bundle(PbrBundle {
                    mesh: bullet_assets.mesh.clone(),
                    material: bullet_assets.material.clone(),
                    transform: Transform::identity().with_translation(global_transform.translation()),
                    ..default()
                })
                .insert(Projectile { damage: 4 })
                .insert(Lifetime{ time_remaining: 5.0 })
                .insert(RigidBody::Dynamic)
                .insert(Ccd::enabled())
                .insert(Velocity {
                    linvel: 100.0 * global_transform.forward(),
                    angvel: Vec3::ZERO,
                })
                .insert(Collider::ball(0.01))
                .insert(CollisionGroups::new(NamedCollisionGroups::Projectile as u32, NamedCollisionGroups::Everything as u32))
                .insert(ColliderMassProperties::Mass(0.1))
                .insert(ActiveEvents::COLLISION_EVENTS);
        }
    }
}

pub fn projectile_impact(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    projectile_query: Query<&Projectile>,
    collider_parent_query: Query<&Parent, With<Collider>>,
    mut health_query: Query<(Entity, &mut Health), With<Zombie>>,
) {
    for collision in collision_events.iter() {
        match collision {
            CollisionEvent::Started(entity1, entity2, _event_flags) => {
                if let Ok(projectile) = projectile_query.get(*entity1) {
                    handle_collision(&mut commands, entity1, entity2, projectile, &collider_parent_query, &mut health_query);
                } else if let Ok(projectile) = projectile_query.get(*entity2) {
                    handle_collision(&mut commands, entity2, entity1, projectile, &collider_parent_query, &mut health_query)
                }
            },
            _ => { }
        }
    }
}

fn handle_collision(
    commands: &mut Commands,
    projectile_entity: &Entity,
    hit_entity: &Entity,
    projectile: &Projectile,
    collider_parent_query: &Query<&Parent, With<Collider>>,
    health_query: &mut Query<(Entity, &mut Health), With<Zombie>>,
) {
    // Try to get the hit parent
    if let Ok(parent) = collider_parent_query.get(*hit_entity) {
        if let Ok((entity, mut health)) = health_query.get_mut(parent.get()) {
            health.current_health -= projectile.damage.min(health.current_health);

            if health.current_health <= 0 { // TODO: Move to separate kill system
                commands.entity(entity).despawn_recursive();
            }
        }
    }
    commands.entity(*projectile_entity).despawn();
}