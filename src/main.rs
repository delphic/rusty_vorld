use bevy::{ prelude::*, input::mouse::MouseMotion };
use bevy_rapier3d::prelude::*;

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
        })
        .add_startup_system(setup)
        .add_system(grab_mouse)
        .add_system(move_camera)
        .add_system(rotate_camera)
        .add_system(update_debug_text);
    }
}

#[derive(Component)]
struct DebugText;

#[derive(Component)]
struct PlayerCamera {
    /// The desired angle around the local x axis
    pitch: f32, 
}

struct GameState {
    cursor_locked: bool,
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

fn move_camera(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
    mut camera_query: Query<&mut Transform, With<PlayerCamera>>,
) {
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

    let velocity = movement_speed * delta_x * local_x + movement_speed * delta_z * local_z;
    let time_delta = time.delta_seconds();

    let shape = Collider::ball(0.25);
    if let Some((_entity, hit)) = rapier_context.cast_shape(
        camera_transform.translation,
        Quat::IDENTITY,
        velocity,
        &shape,
        time_delta,
        QueryFilter::default()
    ) {
        camera_transform.translation += velocity * hit.toi - velocity.normalize() * 0.01;
    } else {
        camera_transform.translation += velocity * time_delta;
    }
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn rotate_camera(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera)>,
) {
    let rotation_speed = 0.1;

    if game_state.cursor_locked && !mouse_motion_events.is_empty() {
        let (mut camera_transform, mut player_camera ) = camera_query.iter_mut().last().unwrap();

        let clamp_angle = std::f32::consts::PI * (0.5 - 10.0 / 180.0); // prevent rotation past 10 degrees

        for event in mouse_motion_events.iter() {
            let scaled_mouse_delta = rotation_speed * time.delta().as_secs_f32() * event.delta;
            let local_x = camera_transform.local_x();
            let pitch = player_camera.pitch;
            let new_pitch = clamp(pitch - scaled_mouse_delta.y, -clamp_angle, clamp_angle);

            camera_transform.rotate_axis(Vec3::Y, -scaled_mouse_delta.x);
            camera_transform.rotate_axis(local_x, new_pitch - pitch);

            player_camera.pitch = new_pitch;
        }
    }
}

fn update_debug_text(
    mut text_query: Query<&mut Text, With<DebugText>>,
    camera_query: Query<&PlayerCamera>,
) {
    let camera_transform = camera_query.iter().last().unwrap();

    for mut text in text_query.iter_mut() {
        text.sections[0].value = format!("{:?}", camera_transform.pitch * 180.0 / std::f32::consts::PI);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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

    // Camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.75, 0.0),
        ..default()
    }).insert(PlayerCamera {
        pitch: 0.0
    });

    // Debug Text
    commands.spawn_bundle(
        TextBundle::from_section("Debug", TextStyle { 
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 64.0,
            color: Color::WHITE,
        })
        .with_text_alignment(TextAlignment::TOP_CENTER)
        .with_style(Style { 
            align_self: AlignSelf::FlexEnd,
            .. default()
        }))
    .insert(DebugText);
}
