use bevy::prelude::*;
use bevy_hanabi::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Copy, Clone)]
pub struct Projectile {
    pub damage: u32,
}

pub struct ProjectileImpactEvent {
    pub projectile: Projectile,
    pub hit_entity: Entity,
}

pub fn setup(
    mut commands: Commands,
    mut effect_assets: ResMut<Assets<EffectAsset>>,
) {
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1.0, 0.97, 0.65, 1.0));
    gradient.add_key(0.75, Vec4::new(0.94, 0.17, 0.07, 0.75));
    gradient.add_key(1.0, Vec4::new(0.94, 0.17, 0.07, 0.0));

    let spawner = Spawner::once(40.0.into(), false);
    let effect_handle = effect_assets.add(EffectAsset {
            name: "Impact".into(),
            capacity: 32768,
            spawner,
            .. default()
        }.init(PositionSphereModifier {
            radius: 0.1,
            speed: 1.2.into(),
            dimension: ShapeDimension::Surface,
            ..default()
        }).init(ParticleLifetimeModifier {
            lifetime: 0.25,
        }).update(AccelModifier {
            accel: Vec3::new(0.0, -3.0, 0.0),
        }).render(BillboardModifier{
        }).render(SizeOverLifetimeModifier {
            gradient: Gradient::constant(Vec2::splat(0.05)),
        }).render(ColorOverLifetimeModifier { gradient }),
    );

    commands.spawn_bundle(ParticleEffectBundle::new(effect_handle).with_spawner(spawner))
        .insert(Name::new("impact effect"));
}

pub fn detect_projectile_impact(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    projectile_query: Query<(&Projectile, &Transform)>,
    mut projectile_event_writer: EventWriter<ProjectileImpactEvent>,
    mut effect_query: Query<(&mut ParticleEffect, &mut Transform), Without<Projectile>>
) {
    for collision in collision_events.iter() {
        let (mut effect, mut effect_transform) = effect_query.single_mut();
        match collision {
            CollisionEvent::Started(entity1, entity2, _event_flags) => {
                if let Ok((projectile, projectile_transform)) = projectile_query.get(*entity1) {
                    projectile_event_writer.send(ProjectileImpactEvent { 
                        projectile: *projectile,
                        hit_entity: *entity2,
                    });

                    effect_transform.translation = projectile_transform.translation;
                    effect.maybe_spawner().unwrap().reset(); // As it's a once - reset spawns new particles

                    commands.entity(*entity1).despawn();
                } else if let Ok((projectile, projectile_transform)) = projectile_query.get(*entity2) {
                    projectile_event_writer.send(ProjectileImpactEvent {
                        projectile: *projectile,
                        hit_entity: *entity1,
                    });

                    effect_transform.translation = projectile_transform.translation;
                    effect.maybe_spawner().unwrap().reset(); // As it's a once - reset spawns new particles

                    commands.entity(*entity2).despawn();
                }
            },
            _ => { }
        }
    }
}