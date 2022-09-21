use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use super::health::Health;
use super::named_collision_groups::*;
use super::zombie::Zombie;

pub struct NpcAssets {
    is_loaded: bool,
    tiny_person: Handle<Scene>,
    // walk_animation: Handle<AnimationClip>,
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let model_handle = asset_server.load("models/tiny_person.gltf#Scene0");
    // let animation_handle = asset_server.load("models/tiny_person.gltf#walk");
    commands.insert_resource(NpcAssets {
        tiny_person: model_handle,
        // walk_animation: animation_handle,
        is_loaded: false,
    });
}

pub fn handle_asset_load(
    mut commands: Commands,
    mut npc_assets: ResMut<NpcAssets>,
    scenes: Res<Assets<Scene>>,
) {
    if !npc_assets.is_loaded {
        if scenes.get(&npc_assets.tiny_person).is_some() {
            npc_assets.is_loaded = true;
            for x in -4..=4 {
                commands
                    .spawn_bundle(SceneBundle {
                    scene: npc_assets.tiny_person.clone(),
                    transform: Transform::from_xyz(x as f32, 0.0, 0.0),
                    ..default()
                    })
                    .insert(Zombie)
                    .insert(Health { max_health: 10, current_health: 10 })
                    .with_children(|child_builder| {
                        // Should probably attempt to get the collision information out of the model, for now, hard code
                        child_builder
                            .spawn_bundle(SpatialBundle { transform: Transform::from_xyz(0.0, 0.5, 0.0), ..default() })
                            .insert(Collider::cuboid(3.0 / 16.0, 0.5, 2.0 / 16.0))
                            .insert(CollisionGroups::new(NamedCollisionGroups::Npc as u32, NamedCollisionGroups::Everything as u32));
                    });
            }
        }
    }
}
