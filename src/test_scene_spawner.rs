use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let green = Color::rgb_u8(0, 90, 20);
    let blue = Color::rgb_u8(0, 40, 90);

    let floor_material = materials.add(StandardMaterial {
        base_color: green,
        perceptual_roughness: 1.0,
        ..default()
    });
    let cube_material = materials.add(blue.into());

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
