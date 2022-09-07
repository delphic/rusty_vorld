use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub mod debug;
pub mod input;
pub mod utils;

mod player_camera;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(VorldPlugin)
        .run();
}

pub struct VorldPlugin;

impl Plugin for VorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState {
            cursor_locked: false,
        }).add_startup_system(setup);
        input::insert_resources(app);

        app.add_system(grab_mouse);
        input::add_systems(app);
        player_camera::add_systems(app);
    }
}

pub struct GameState {
    pub cursor_locked: bool,
}

fn grab_mouse(
    mut windows: ResMut<Windows>,
    mut game_state: ResMut<GameState>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let window = windows.get_primary_mut().unwrap();
    if mouse_button_input.just_pressed(MouseButton::Left) {
        window.set_cursor_visibility(false);
        window.set_cursor_lock_mode(true);
        game_state.cursor_locked = true;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        window.set_cursor_visibility(true);
        window.set_cursor_lock_mode(false);
        game_state.cursor_locked = false;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let green = Color::rgb_u8(0, 90, 20);
    let blue = Color::rgb_u8(0, 40, 90);

    let floor_material = materials.add(StandardMaterial { 
        base_color: green,
        perceptual_roughness: 1.0,
        .. default()
    });
    let cube_material = materials.add(blue.into());

    let floor_mesh = meshes.add(Mesh::from(shape::Plane { size: 32.0 }));
    let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    // Would like some good old gourd shading really but for now PBR as bevy comes with it
    commands.spawn_bundle(PbrBundle {
        mesh: floor_mesh,
        material: floor_material,
        ..default()
    }).insert(Collider::cuboid(16.0, 0.001, 16.0));

    for i in 0..4 {
        commands.spawn_bundle(PbrBundle {
            mesh: cube_mesh.clone(),
            material: cube_material.clone(),
            transform: Transform::from_xyz(8.0 * (i as f32 - 1.5), 0.5, 8.0),
            ..default()
        }).insert(Collider::cuboid(0.5, 0.5, 0.5));
        commands.spawn_bundle(PbrBundle {
            mesh: cube_mesh.clone(),
            material: cube_material.clone(),
            transform: Transform::from_xyz(8.0 * (i as f32 - 1.5), 0.5, -8.0),
            ..default()
        }).insert(Collider::cuboid(0.5, 0.5, 0.5));
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
