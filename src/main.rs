use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;

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
        app
            .insert_resource(GameState { cursor_locked: false })
            .add_startup_system(setup)
            .add_system(grab_mouse)
            .add_system(move_camera)
            .add_system(rotate_camera);
    }
}

struct GameState {
    cursor_locked: bool
}

fn grab_mouse(
    mut windows: ResMut<Windows>,
    mut game_state: ResMut<GameState>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>
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

fn move_camera(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera>>) {

    let movement_speed = 5.0;
    let mut camera_transform = camera_query.iter_mut().last().unwrap();
    let mut delta_x = 0.0;
    if keyboard_input.pressed(KeyCode::A) {
        delta_x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) {
        delta_x += 1.0;
    }

    let mut delta_z = 0.0;
    if keyboard_input.pressed(KeyCode::W) {
        delta_z -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::S) {
        delta_z += 1.0;
    }

    let local_x = camera_transform.local_x();
    let local_z = camera_transform.local_z();

    camera_transform.translation += movement_speed * time.delta().as_secs_f32() * delta_x * local_x;
    camera_transform.translation += movement_speed * time.delta().as_secs_f32() * delta_z * local_z;
}

fn rotate_camera(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_query: Query<&mut Transform, With<Camera>>) {

    let rotation_speed = 0.1;

    if game_state.cursor_locked && !mouse_motion_events.is_empty() {
        let mut camera_transform = camera_query.iter_mut().last().unwrap();

        for event in mouse_motion_events.iter() {
            let delta = rotation_speed * time.delta().as_secs_f32() * event.delta;
            let local_x = camera_transform.local_x();
            camera_transform.rotate_axis(Vec3::Y, -delta.x);
            camera_transform.rotate_axis(local_x, -delta.y);
        }
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
           transform: Transform::from_xyz(8.0 * (i as f32 - 1.5), 0.5, 8.0),
           .. default()
       });
       commands.spawn_bundle(PbrBundle {
           mesh: cube_mesh.clone(),
           material: cube_material.clone(),
           transform: Transform::from_xyz(8.0 * (i as f32 - 1.5), 0.5, -8.0),
           .. default()
       });
    }

    // Lighting
    commands.insert_resource(AmbientLight::default());

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

    // Camera
    commands.spawn_bundle(Camera3dBundle {
       transform: Transform::from_xyz(0.0, 1.75, 0.0),
       .. default()
    });
}