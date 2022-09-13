use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::input::PlayerInput;
use super::utils;

#[derive(Component)]
struct PlayerCamera {
    /// The desired angle around the local x axis, -π/2 -> π/2
    pitch: f32,
    /// The desired angle around the global y axis, 0 -> 2π
    yaw: f32,
}

pub fn add_systems(app: &mut App) {
    app.add_startup_system(setup)
        .add_system(rotate_camera)
        .add_system(move_camera);
}

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(8.0, 1.75, -8.0),
            ..default()
        })
        .insert(PlayerCamera {
            pitch: 0.0,
            yaw: std::f32::consts::PI,
        });
}

fn move_camera(
    time: Res<Time>,
    player_input: Res<PlayerInput>,
    rapier_context: Res<RapierContext>,
    mut camera_query: Query<&mut Transform, With<PlayerCamera>>,
) {
    let movement_speed = 10.0;
    let mut camera_transform = camera_query.iter_mut().last().unwrap();

    let local_x = camera_transform.local_x();
    let local_z = camera_transform.local_z();

    let velocity = movement_speed * player_input.movement_direction.x * local_x
        + movement_speed * player_input.movement_direction.z * local_z;

    if velocity.length_squared() > 0.0 {
        let time_delta = time.delta_seconds();

        let collider_radius = 0.25;
        let shape = Collider::ball(collider_radius);
        let start_translation = camera_transform.translation;

        let skin_depth = 0.01;
        let velocity_direction = velocity.normalize();
        let velocity_magnitude = velocity.length();

        let mut collision_disabled = false;

        if let Some((_, hit)) = rapier_context.cast_shape(
            camera_transform.translation,
            Quat::IDENTITY,
            velocity_direction,
            &shape,
            time_delta * velocity_magnitude + skin_depth,
            QueryFilter::default(),
        ) {
            if hit.toi == 0.0 {
                // Already overlapping - should only happen if teleported or spawned inside collider
                warn!("Started camera movement already overlapping, collision disabled");
                camera_transform.translation += velocity * time_delta;
                collision_disabled = true;
            } else {
                // Desired movement collides, attempt to slide along surface
                // NOTE: Casting in velocity direction means time of impact is in fact distance to impact
                let close_distance = hit.toi - skin_depth;
                camera_transform.translation += velocity_direction * close_distance;
                // ^^ This can be negative and will attempt to move the camera away before sliding along the surface

                let stop_time = close_distance / velocity_magnitude;
                let time_remainder = time_delta - stop_time;
                let velocity_remainder = velocity * time_remainder / time_delta;
                let slide_velocity =
                    velocity_remainder - velocity_remainder.dot(hit.normal1) * hit.normal1;
                let slide_velocity_direction = slide_velocity.normalize();
                let slide_velocity_magnitude = slide_velocity.length();

                if let Some((_, second_hit)) = rapier_context.cast_shape(
                    camera_transform.translation,
                    Quat::IDENTITY,
                    slide_velocity_direction,
                    &shape,
                    time_remainder * slide_velocity_magnitude + skin_depth,
                    QueryFilter::default(),
                ) {
                    // slide also collides, attempt to one further slide in direction perpenticular to both hit normals
                    let second_slide_direction = hit.normal1.cross(second_hit.normal1);

                    let close_distance = second_hit.toi - skin_depth;
                    camera_transform.translation += slide_velocity_direction * close_distance;

                    let time_delta = time_remainder;
                    let stop_time = close_distance / slide_velocity_magnitude;
                    let time_remainder = time_delta - stop_time;
                    let velocity_remainder = slide_velocity * time_remainder / time_delta;
                    let slide_velocity =
                        velocity_remainder.dot(second_slide_direction) * second_slide_direction;

                    if rapier_context
                        .cast_shape(
                            camera_transform.translation,
                            Quat::IDENTITY,
                            slide_velocity.normalize(),
                            &shape,
                            time_remainder * slide_velocity.length() + skin_depth,
                            QueryFilter::default(),
                        )
                        .is_none()
                    {
                        // Only move if there are no collisions only second slide axis, else only move up to second contact
                        camera_transform.translation += slide_velocity * time_remainder;
                    }
                } else {
                    camera_transform.translation += slide_velocity * time_remainder;
                }
            }
        } else {
            camera_transform.translation += velocity * time_delta;
        }

        if !collision_disabled {
            rapier_context.intersections_with_shape(camera_transform.translation, Quat::IDENTITY, &shape, QueryFilter::default(), |_| {
                // cast_shape sometimes lies about there being no collision due to float precision issues,
                // so check for intersections and if found restore to starting position
                warn!("Camera shape found to intersect world collider after movement, restoring to last valid position");
                camera_transform.translation = start_translation;
                false
            });
        }
    }
}

fn rotate_camera(
    time: Res<Time>,
    player_input: Res<PlayerInput>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera)>,
) {
    let rotation_speed = 0.1; // TODO: degrees = dots * 0.022

    if let Some((mut camera_transform, mut player_camera)) = camera_query.iter_mut().last() {
        // prevent rotation past 10 degrees towards vertical
        let clamp_angle = std::f32::consts::PI * (0.5 - 10.0 / 180.0);

        let scaled_mouse_delta = rotation_speed * time.delta_seconds() * player_input.mouse_motion;

        let yaw = (player_camera.yaw - scaled_mouse_delta.x) % (2.0 * std::f32::consts::PI);
        let pitch = utils::clamp(
            player_camera.pitch - scaled_mouse_delta.y,
            -clamp_angle,
            clamp_angle,
        );

        camera_transform.rotation = Quat::IDENTITY;
        camera_transform.rotate_axis(Vec3::Y, yaw);
        let local_x = camera_transform.local_x();
        camera_transform.rotate_axis(local_x, pitch);

        player_camera.yaw = yaw;
        player_camera.pitch = pitch;
    }
}
