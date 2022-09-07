use bevy::{ prelude::*, input::mouse::MouseMotion };
use bevy_rapier3d::prelude::*;
mod debug;

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
        .add_system(rotate_camera);
    }
}

#[derive(Component)]
struct PlayerCamera {
    /// The desired angle around the local x axis, -π/2 -> π/2
    pitch: f32,
    /// The desired angle around the global y axis, 0 -> 2π
    yaw: f32,
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
    let start_translation = camera_transform.translation;

    if let Some((_, hit)) = rapier_context.cast_shape(
        camera_transform.translation,
        Quat::IDENTITY,
        velocity,
        &shape,
        time_delta + 0.001,
        QueryFilter::default()
    ) {
        if hit.toi == 0.0 {
            panic!("Started camera movement already overlapping");
        } else {
            let stop_time = f32::max(hit.toi - 0.001, 0.0); // adjusted to allow for a minimum separation - TODO: should do this by distance not time
            camera_transform.translation += velocity * stop_time;
            
            let time_remainder = time_delta - stop_time;
            let velocity_remainder = velocity * time_remainder / time_delta;
            let slide_velocity = velocity_remainder - velocity_remainder.dot(hit.normal1) * hit.normal1;
            if rapier_context.cast_shape(
                camera_transform.translation,
                Quat::IDENTITY,
                slide_velocity,
                &shape,
                time_remainder + 0.001,
                QueryFilter::default()).is_none() {
                    camera_transform.translation += slide_velocity * time_remainder;
            } // TODO: Attempt move up to new contact and second slide axis before giving up
        }
    } else if velocity.length_squared() > 0.0 {
        camera_transform.translation += velocity * time_delta;
    }

    rapier_context.intersections_with_shape(camera_transform.translation, Quat::IDENTITY, &shape, QueryFilter::default(), |_| {
        // Cast Shape sometimes lies about there being no collision due to float precision issues,
        // so check for intersections and if found restore to starting position
        debug::log_error("Camera shape found to intersect world collider after movement, restoring to last valid position");
        camera_transform.translation = start_translation;
        false
    });
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
        if let Some((mut camera_transform, mut player_camera)) = camera_query.iter_mut().last() {
            // prevent rotation past 10 degrees towards vertical
            let clamp_angle = std::f32::consts::PI * (0.5 - 10.0 / 180.0); 

            let mut yaw = player_camera.yaw;
            let mut pitch = player_camera.pitch;

            for event in mouse_motion_events.iter() {
                let scaled_mouse_delta = rotation_speed * time.delta_seconds() * event.delta;

                yaw = (yaw - scaled_mouse_delta.x) % (2.0 * std::f32::consts::PI);
                pitch = clamp(pitch - scaled_mouse_delta.y, -clamp_angle, clamp_angle);
            }
            
            camera_transform.rotation = Quat::IDENTITY;
            camera_transform.rotate_axis(Vec3::Y, yaw);
            let local_x = camera_transform.local_x();
            camera_transform.rotate_axis(local_x, pitch);

            player_camera.yaw = yaw;
            player_camera.pitch = pitch;
        }
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

    // Camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.75, 0.0),
        ..default()
    }).insert(PlayerCamera {
        pitch: 0.0,
        yaw: 0.0,
    });
}
