use bevy::{prelude::*, render::texture::ImageSampler};
use bevy_rapier3d::prelude::*;
use super::mesher;

pub struct VorldAtlas {
    image_handle: Handle<Image>,
    loaded: bool,
}

pub fn asset_load_handler(
    mut image_assets: ResMut<Assets<Image>>,
    mut atlas: ResMut<VorldAtlas>
) {
    // NOTE: Tried using EventReader<AssetEvent<Image>> to change this on AssetEvent::Created
    // however it had no effect on how the image was rendered - bevy bug?
    if !atlas.loaded {
        if let Some(texture) = image_assets.get_mut(&atlas.image_handle) {
            texture.sampler_descriptor = ImageSampler::nearest();
            // TODO: See if it's possible to have nearest mag filter and linear min filter 
            // (unlike in WebGL, which ignores the request for nearest mag filter)
            atlas.loaded = true;
        }
    }
}

pub fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let green = Color::rgb_u8(0, 90, 20);
    let blue = Color::rgb_u8(0, 40, 90);

    let tile_atlas = asset_server.load("images/atlas.png");

    let atlas_material = materials.add(StandardMaterial {
        base_color_texture: Some(tile_atlas.clone()),
        ..default()
    });
    let floor_material = materials.add(StandardMaterial {
        base_color: green,
        perceptual_roughness: 1.0,
        ..default()
    });
    let cube_material = materials.add(blue.into());

    let tile_count = 23;
    let atlas_definition = VorldAtlas {
        image_handle: tile_atlas.clone(),
        loaded: false,
    };
    commands.insert_resource(atlas_definition);

    let floor_mesh = meshes.add(Mesh::from(shape::Plane { size: 32.0 }));
    let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    // Would like some good old gourd shading really but for now PBR as bevy comes with it
    commands
        .spawn_bundle(PbrBundle {
            mesh: floor_mesh,
            material: floor_material,
            ..default()
        })
        .insert(Collider::cuboid(16.0, 0.001, 16.0));

    for i in 0..tile_count {
        let tile_mesh = meshes.add(mesher::build_tile(i, tile_count));
        commands.spawn_bundle(PbrBundle {
            mesh: tile_mesh,
            material: atlas_material.clone(),
            transform: Transform::from_xyz(i as f32 * 1.5 - 0.5 * tile_count as f32, 1.0, -4.0),
            ..default()
        });
    }

    for i in 0..4 {
        commands
            .spawn_bundle(PbrBundle {
                mesh: cube_mesh.clone(),
                material: cube_material.clone(),
                transform: Transform::from_xyz(8.0 * (i as f32 - 1.5), 0.5, 8.0),
                ..default()
            })
            .insert(Collider::cuboid(0.5, 0.5, 0.5));
        commands
            .spawn_bundle(PbrBundle {
                mesh: cube_mesh.clone(),
                material: cube_material.clone(),
                transform: Transform::from_xyz(8.0 * (i as f32 - 1.5), 0.5, -8.0),
                ..default()
            })
            .insert(Collider::cuboid(0.5, 0.5, 0.5));
    }

    // Lighting
    commands.insert_resource(AmbientLight::default());

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgba_u8(230, 220, 200, 255),
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 10.0, 0.0).with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -45.0,
            -20.0,
            0.0,
        )),
        ..default()
    });
}
