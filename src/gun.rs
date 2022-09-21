use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::lifetime::*;
use super::named_collision_groups::*;
use super::input::PlayerInput;
use super::projectile::Projectile;

pub struct BulletMeshMaterial {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>
}

#[derive(Component)]
pub struct Muzzle;

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
    transform_query: Query<&GlobalTransform, With<Muzzle>>,
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
                    linvel: 500.0 * global_transform.forward(),
                    angvel: Vec3::ZERO,
                })
                .insert(Collider::ball(0.01))
                .insert(CollisionGroups::new(NamedCollisionGroups::Projectile as u32, NamedCollisionGroups::Everything as u32))
                .insert(ColliderMassProperties::Mass(0.1))
                .insert(ActiveEvents::COLLISION_EVENTS);
        }
    }
}

