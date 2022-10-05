use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::hit_flash::HitFlashSupport;
use super::health::Health;
use super::named_collision_groups::*;
use super::zombie::Zombie;
use super::utils;

pub struct NpcAssets {
    is_loaded: bool,
    tiny_person: Handle<Scene>,
    pub walk_animation: Handle<AnimationClip>,
}

#[derive(Component)]
pub struct Npc {
    pub animation_player_entity: Option<Entity>,
}

#[derive(Component)]
pub struct FindAnimationPlayerRequest;

#[derive(Component)]
pub struct CloneModelMaterialsRequest;

pub struct NpcSpawnerPlugin;

impl Plugin for NpcSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        app.add_system(handle_asset_load);
        app.add_system(handle_find_animation_player_request);
        app.add_system(handle_clone_model_materials_request);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let model_handle = asset_server.load("models/tiny_person.gltf#Scene0");
    let animation_handle = asset_server.load("models/tiny_person.gltf#Animation0");
    // ^^ If we want to get the animations by name we need to load the gtlf and enumerate through it's structure
    commands.insert_resource(NpcAssets {
        tiny_person: model_handle,
        walk_animation: animation_handle,
        is_loaded: false,
    });
}

fn handle_asset_load(
    mut commands: Commands,
    mut npc_assets: ResMut<NpcAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
                    .insert(Npc { animation_player_entity: None })
                    .insert(FindAnimationPlayerRequest)
                    .insert(CloneModelMaterialsRequest)
                    .insert(Zombie::new())
                    .insert(Health::new(10))
                    .with_children(|child_builder| {
                        // Should probably attempt to get the collision information out of the model, for now, hard code
                        child_builder
                            .spawn_bundle(SpatialBundle { transform: Transform::from_xyz(0.0, 0.5, 0.0), ..default() })
                            .insert(Collider::cuboid(3.0 / 16.0, 0.5, 2.0 / 16.0))
                            .insert(CollisionGroups::new(NamedCollisionGroups::Npc as u32, NamedCollisionGroups::Everything as u32));
                    });
            }

            let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
            let blue = Color::rgb_u8(0, 40, 90);
            let cube_material = materials.add(blue.into());
            commands
                .spawn_bundle(PbrBundle {
                    mesh: cube_mesh.clone(),
                    material: cube_material.clone(),
                    transform: Transform::from_xyz(8.0, 0.5, 8.0),
                    ..default()
                })
                .insert(Npc { animation_player_entity: None })
                .insert(Health { max_health: 100, current_health: 100 })
                .insert(HitFlashSupport { material: cube_material.clone(), base_color: blue, flash_color: Color::RED  })
                .insert(Collider::cuboid(0.5, 0.5, 0.5))
                .insert(CollisionGroups::new(NamedCollisionGroups::Npc as u32, NamedCollisionGroups::Everything as u32));
        }
    }
}

fn handle_find_animation_player_request(
    mut commands: Commands, 
    mut request_query: Query<(Entity, &mut Npc), With<FindAnimationPlayerRequest>>,
    hierarchy_query: Query<(&Children, Option<&AnimationPlayer>)>,
    animation_player_query: Query<&AnimationPlayer>,
) {
    for (entity, mut npc) in request_query.iter_mut() {
        if let Ok((children, _)) = hierarchy_query.get(entity) {
            npc.animation_player_entity = utils::find_child_entity_with_component(children, &hierarchy_query, &animation_player_query);
        }
        commands.entity(entity).remove::<FindAnimationPlayerRequest>();
    }
}

fn handle_clone_model_materials_request(
    mut commands: Commands, 
    mut request_query: Query<Entity, With<CloneModelMaterialsRequest>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    hierarchy_query: Query<(&Children, Option<&Handle<StandardMaterial>>)>,
    material_query: Query<&Handle<StandardMaterial>>,
) {
    for entity in request_query.iter_mut() {
        if let Ok((children, _)) = hierarchy_query.get(entity) {
            let mut material_entities = Vec::new();
            utils::find_children_with_component(&mut material_entities, children, &hierarchy_query, &material_query);

            if !material_entities.is_empty() {
                let mut cloned_material_handle_option = None;
                let mut base_color = Color::WHITE;

                if let Some(material_entity) = material_entities.first() {
                    if let Ok(material_handle) = material_query.get(*material_entity) {
                        let mut cloned_material_option = None;
                        if let Some(material) = materials.get(material_handle) {
                            cloned_material_option = Some(material.clone());
                            base_color = material.base_color;
                        } else {
                            warn!("Unable to find model material in assets resource")
                        }

                        if let Some(new_material) = cloned_material_option {
                            cloned_material_handle_option = Some(materials.add(new_material));
                        }
                    }
                }

                if let Some(cloned_material_handle) = cloned_material_handle_option {
                    commands.entity(entity)
                        .insert(HitFlashSupport {
                            material: cloned_material_handle.clone(),
                            base_color: base_color,
                            flash_color: Color::RED,
                        });

                    // Replace the materials on the model with this clone
                    // NOTE: Assumes single material per model
                    for material_entity in material_entities.iter() {
                        commands.entity(*material_entity)
                            .remove::<Handle<StandardMaterial>>()
                            .insert(cloned_material_handle.clone());
                    }
                }
            }
        }
        commands.entity(entity).remove::<CloneModelMaterialsRequest>();
    }
}
