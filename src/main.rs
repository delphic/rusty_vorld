use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(VorldPlugin)
        .run();
}

pub struct VorldPlugin;

impl Plugin for VorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>)
{
     let green = Color::rgb_u8(0, 90, 20);
     let blue = Color::rgb_u8(0, 40, 90);

     let floor_material = materials.add(green.into());
     let cube_material = materials.add(blue.into());

     let floor_mesh = meshes.add(Mesh::from(shape::Plane { size: 32.0 }));
     let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

     // Would like some good old gourd shading really but for now PBR as bevy comes with it
     commands.spawn_bundle(PbrBundle { 
        mesh: floor_mesh,
        material: floor_material,
        .. default()
     });

     for i in 0..4 {
        commands.spawn_bundle(PbrBundle {
            mesh: cube_mesh.clone(),
            material: cube_material.clone(),
            transform: Transform::from_xyz(8.0 * (i as f32 - 2.0), 0.5, 8.0),
            .. default()
        });
        commands.spawn_bundle(PbrBundle {
            mesh: cube_mesh.clone(),
            material: cube_material.clone(),
            transform: Transform::from_xyz(8.0 * (i as f32 - 2.0), 0.5, -8.0),
            .. default()
        });
     }

     // Direction Light (Q: how ambient?)
     commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgba_u8(230, 220, 200, 255),
            illuminance: 10000.0,
            shadows_enabled: true,
            .. default()
        },
        transform: Transform::from_xyz(0.0, 10.0, 0.0).with_rotation(Quat::from_euler(EulerRot::XYZ, -45.0, -20.0, 0.0)),
        .. default()
     });

     commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.0, 0.0),
        .. default()
     });
}