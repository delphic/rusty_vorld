use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::gun;
use super::player_input::PlayerInput;
use super::named_collision_groups::*;
use super::smoothed_follow::SmoothedFollow;
use super::utils;

#[derive(Component)]
pub struct Player {
    velocity: Vec3,
    is_grounded: bool,
    is_crouched: bool,
}

#[derive(Component)]
pub struct PlayerCamera {
    target: Entity,
    offset: Vec3,
    /// The desired angle around the local x axis, -π/2 -> π/2
    pitch: f32,
    /// The desired angle around the global y axis, 0 -> 2π
    yaw: f32,
}

#[derive(Component)]
pub struct AttachMuzzleRequest {
    entity: Entity
}

struct PlayerMovementConfig {
    acceleration: f32,
    air_acceleration: f32,
    max_run_speed: f32,
    max_air_movement_speed: f32,
    stop_speed: f32,
    jump_delta_v: f32,
    crouch_jump_delta_v: f32,
    acceleration_due_to_gravity: f32,
}

struct PlayerCollisionConfig {
    player_standing_half_height: f32,
    player_crouched_half_height: f32,
    skin_depth: f32,
    collider_radius: f32,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(attach_muzzle)
            .add_system(move_player)
            .add_system(update_look.after(move_player));
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let player_spawn_point = Vec3::new(8.0, 1.0, -8.0);
    let camera_offset = Vec3::new(0.0, 1.25, 0.0);

    let movement_config = PlayerMovementConfig {
        acceleration: 80.0,
        air_acceleration: 10.0,
        max_run_speed: 5.5,
        max_air_movement_speed: 4.0,
        stop_speed: 1.5,
        jump_delta_v: 7.5,
        crouch_jump_delta_v: 6.0,
        acceleration_due_to_gravity: 2.0 * 9.8,
    };

    let collider_config = PlayerCollisionConfig {
        player_standing_half_height: 1.0,
        player_crouched_half_height: 0.5,
        skin_depth: 0.01,
        collider_radius: 0.25,
    };

    commands.insert_resource(movement_config);
    commands.insert_resource(collider_config);

    let player_entity =  commands
        .spawn()
        .insert_bundle(SpatialBundle { transform: Transform::identity().with_translation(player_spawn_point), ..default() })
        .insert(Player {
            velocity: Vec3::ZERO,
            is_grounded: false,
            is_crouched: false,
        }).id();
    
    let camera_entity = commands.spawn_bundle(SpatialBundle::default())
        .insert_bundle(Camera3dBundle { 
            transform: Transform::identity().with_translation(player_spawn_point + camera_offset),
            ..default()
        }).insert(PlayerCamera {
            target: player_entity,
            offset: camera_offset,
            pitch: 0.0,
            yaw: std::f32::consts::PI,
        }).id();
    
    let gun_pivot = Vec3::new(0.125, -0.125, -0.25);
    let model_entity = commands.spawn_bundle(SceneBundle {
        scene: asset_server.load("models/rifle.gltf#Scene0"),
        transform: Transform::identity().with_translation(player_spawn_point + camera_offset + gun_pivot),
        ..default()
    }).insert(SmoothedFollow {
        target: camera_entity,
        translation_offset: gun_pivot,
        translation_rate: 0.9,
        rotation_rate: 0.5,
    }).id();
    
    commands.spawn().insert(AttachMuzzleRequest { entity: model_entity });
}

fn attach_muzzle(
    mut commands: Commands,
    request_query: Query<(Entity, &AttachMuzzleRequest)>,
    traversal_query: Query<(&Children, Option<&Name>)>,
) {
    for (entity, request) in request_query.iter() {
        if let Ok((children, _)) = traversal_query.get(request.entity) {
            if let Some(muzzle_entity) = utils::find_child_with_name("muzzle", children, &traversal_query) {
                commands.entity(muzzle_entity).insert(gun::Muzzle);
            } else {
                warn!("Unable to find child entity with name muzzle, attaching to root");
                commands.entity(request.entity).insert(gun::Muzzle);
            }
            commands.entity(entity).despawn();
        }
    }
}

fn move_player(
    time: Res<Time>,
    movement_config: Res<PlayerMovementConfig>,
    collider_config: Res<PlayerCollisionConfig>,
    mut player_input: ResMut<PlayerInput>,
    rapier_context: Res<RapierContext>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
) {
    let (mut player_transform, mut player) = player_query.iter_mut().last().unwrap();

    let terrain_filter = QueryFilter::only_fixed().groups(InteractionGroups::new(NamedCollisionGroups::Everything as u32, NamedCollisionGroups::Terrain as u32));

    // Determine crouch / uncrouch
    if !player.is_crouched && player_input.crouch_requested {
        player.is_crouched = true;
    } else if player.is_crouched && !player_input.crouch_requested {
        player.is_crouched = false;
        // Ensure there is space to stand up!
        let shape = Collider::capsule_y(collider_config.player_standing_half_height - collider_config.skin_depth - collider_config.collider_radius, collider_config.collider_radius);
        rapier_context.intersections_with_shape(
            player_transform.translation + collider_config.player_standing_half_height * Vec3::Y,
            Quat::IDENTITY,
            &shape,
            terrain_filter,
            |_| {
                player.is_crouched = true;
                false
            }
        );
    }
    let half_player_height = match player.is_crouched { false =>  collider_config.player_standing_half_height, true => collider_config.player_crouched_half_height };
    let shape = Collider::capsule_y(half_player_height - collider_config.skin_depth - collider_config.collider_radius, collider_config.collider_radius);

    let local_x = player_transform.local_x();
    let local_z = player_transform.local_z();
    
    // Project to x/z plane (arguably should project onto plane of ground if grounded 0 if normal is ~45 degrees of vertical)
    let local_x = Vec3::new(local_x.x, 0.0, local_x.z).normalize();
    let local_z = Vec3::new(local_z.x, 0.0, local_z.z).normalize();

    let time_delta = time.delta_seconds();

    // Transform movement input into world_space 
    let input_vector = player_input.movement_direction.x * local_x + player_input.movement_direction.z * local_z;

    if !player.is_grounded && player.velocity.length_squared() > 0.0 {
        // Apply Drag 
        let air_speed = player.velocity.length();
        let drag_delta_v = air_speed * air_speed * 1.225 * time_delta / 200.0;
        // Assumes in air and mass of 100kg, drag coefficient of ~1 and surface area ~1
        
        if air_speed < drag_delta_v { // Happens at around air_speed of 99 m/s
            player.velocity = Vec3::ZERO;
            // If we wanted to support drag at extremely high speeds properly would need to average drag across the frame, rather than instanteous maximum
        } else {
            player.velocity *= (air_speed - drag_delta_v) / air_speed;
        }
    }

    let player_xz_velocity = Vec3::new(player.velocity.x, 0.0, player.velocity.z);
    // should be on movement plane see comment above about local x/z plane

    let mut target_velocity = player_xz_velocity;
    if player.is_grounded {
        calculate_target_grounded_velocity(&movement_config, player_xz_velocity, &player_input, &mut target_velocity, time_delta, input_vector);
    } else {
        calculate_target_air_velocity(player_xz_velocity, &movement_config, time_delta, input_vector, &mut target_velocity);
    }

    let start_translation = player_transform.translation;
    move_xz(&mut player_transform, target_velocity, &rapier_context, half_player_height, &shape, time_delta, collider_config.skin_depth, terrain_filter);

    // Handle requested y-movement / movement due to gravity
    let vertical_velocity = match player_input.jump_requested {
        true => { 
            player_input.jump_requested = false;
            match player.is_crouched {
                true => movement_config.crouch_jump_delta_v,
                false => movement_config.jump_delta_v,
            }
            // ^^ Air jump style - arrest all vertical momentum 
        },
        false => player.velocity.y - movement_config.acceleration_due_to_gravity * time_delta,
    };

    let direction = match vertical_velocity > 0.0 {
        true => Vec3::Y,
        false => Vec3::NEG_Y,
    };

    move_y(vertical_velocity, &mut player_transform, rapier_context, half_player_height, direction, shape, time_delta, collider_config.skin_depth, terrain_filter, &mut player);

    player.velocity = (player_transform.translation - start_translation) / time_delta;
}

fn calculate_target_grounded_velocity(
    movement_config: &Res<PlayerMovementConfig>,
    player_xz_velocity: Vec3,
    player_input: &ResMut<PlayerInput>,
    target_velocity: &mut Vec3,
    time_delta: f32,
    input_vector: Vec3
) {
    let max_movement_speed = movement_config.max_run_speed;
    // May change in future to allow sprint & walk
    let max_movement_speed_sqr = max_movement_speed * max_movement_speed;
    let speed_sqr = player_xz_velocity.length_squared();
    let is_sliding = speed_sqr > max_movement_speed_sqr + 0.001;
    let any_input = player_input.movement_direction.length_squared() > 0.0;
    if is_sliding {
        // Apply linear slowing force
        // Proportional to v can quickly result at velocity being negated at high speeds 
        *target_velocity *= 1.0 - (5.0 * time_delta).min(1.0);

        // Only allow deceleration if moving faster than max movment speed
        if player_xz_velocity.x.is_sign_positive() != input_vector.x.is_sign_positive() {
            target_velocity.x += movement_config.acceleration * time_delta * input_vector.x;
        }
        if player_xz_velocity.z.is_sign_positive() != input_vector.z.is_sign_positive() {
            target_velocity.z += movement_config.acceleration * time_delta * input_vector.z;
        }
    } else if any_input {
        // Apply slow if input in opposite direction to velocity for faster change of direction
        if player_xz_velocity.x.is_sign_positive() != input_vector.x.is_sign_positive() {
            target_velocity.x *= 1.0 - (2.5 * speed_sqr.sqrt() * time_delta).min(1.0);
        } 
        if player_xz_velocity.z.is_sign_positive() != input_vector.z.is_sign_positive() {
            target_velocity.z *= 1.0 - (2.5 * speed_sqr.sqrt() * time_delta).min(1.0);
        }
        *target_velocity += movement_config.acceleration * time_delta * input_vector;
    
        if target_velocity.length_squared() > max_movement_speed_sqr {
            *target_velocity = max_movement_speed * target_velocity.normalize();
        }
    } else {
        if speed_sqr < movement_config.stop_speed * movement_config.stop_speed {
            *target_velocity = Vec3::ZERO;
        } else {
            *target_velocity *= (2.5 * speed_sqr.sqrt() * time_delta).min(1.0)
        }
    }
}

fn calculate_target_air_velocity(
    player_xz_velocity: Vec3,
    movement_config: &Res<PlayerMovementConfig>,
    time_delta: f32,
    input_vector: Vec3,
    target_velocity: &mut Vec3
) {
    let target_x = player_xz_velocity.x + movement_config.air_acceleration * time_delta * input_vector.x;
    let target_z = player_xz_velocity.z + movement_config.air_acceleration * time_delta * input_vector.z;
    let max_air_movement_speed_sqr = movement_config.max_air_movement_speed * movement_config.max_air_movement_speed;
    let target_air_speed_sqr = target_x * target_x + target_z * target_z;
    let can_accelerate = target_air_speed_sqr < max_air_movement_speed_sqr;
    if can_accelerate || target_x.abs() < player_xz_velocity.x.abs() {
        target_velocity.x = target_x;
    }
    if can_accelerate || target_z.abs() < player_xz_velocity.z.abs() {
        target_velocity.z = target_z;
    }
    if !(target_velocity.x == target_x && target_velocity.z == target_z) {
        // Must be above max air movement speed, and not trying to decelerate in both axes
        let redirect_threshold_speed_sqr = (movement_config.max_run_speed * movement_config.max_run_speed).max(max_air_movement_speed_sqr);
        let current_air_speed_sqr = target_velocity.length_squared(); 
        if current_air_speed_sqr < redirect_threshold_speed_sqr {
            // allow redirection of the direction of air movement if below redirect threshold
            *target_velocity = (current_air_speed_sqr.sqrt() / target_air_speed_sqr.sqrt()) * Vec3::new(target_x, 0.0, target_z);
        }
    }
}

fn move_xz(
    player_transform: &mut Mut<Transform>,
    target_velocity: Vec3,
    rapier_context: &Res<RapierContext>,
    half_player_height: f32,
    shape: &Collider,
    time_delta: f32,
    skin_depth: f32,
    collision_filter: QueryFilter,
) {
    let mut started_overlapping = false;
    let reset_position = player_transform.translation;
    // Character Controller Move
    if target_velocity.length_squared() > 0.0 {
        let velocity_direction = target_velocity.normalize();
        let velocity_magnitude = target_velocity.length();

        if let Some((_, hit)) = rapier_context.cast_shape(
            player_transform.translation + half_player_height * Vec3::Y,
            Quat::IDENTITY,
            velocity_direction,
            shape,
            time_delta * velocity_magnitude + skin_depth,
            collision_filter,
        ) {
            if hit.toi == 0.0 {
                // Already overlapping - should only happen if teleported or spawned inside collider
                warn!("Started movement already overlapping");
                player_transform.translation += target_velocity * time_delta;
                started_overlapping = true;
            } else {
                // Desired movement collides, attempt to slide along surface
                // NOTE: Casting in velocity direction means time of impact is in fact distance to impact
                let close_distance = hit.toi - skin_depth;
                player_transform.translation += velocity_direction * close_distance;
                // ^^ This can be negative and will attempt to move the camera away before sliding along the surface

                let stop_time = close_distance / velocity_magnitude;
                let time_remainder = time_delta - stop_time;
                let velocity_remainder = target_velocity * time_remainder / time_delta;
                let slide_velocity =
                    velocity_remainder - velocity_remainder.dot(hit.normal1) * hit.normal1;
                let slide_velocity_direction = slide_velocity.normalize();
                let slide_velocity_magnitude = slide_velocity.length();

                if let Some((_, second_hit)) = rapier_context.cast_shape(
                    player_transform.translation + half_player_height * Vec3::Y,
                    Quat::IDENTITY,
                    slide_velocity_direction,
                    shape,
                    time_remainder * slide_velocity_magnitude + skin_depth,
                    collision_filter,
                ) {
                    // slide also collides, attempt to one further slide in direction perpenticular to both hit normals
                    let second_slide_direction = hit.normal1.cross(second_hit.normal1);

                    let close_distance = second_hit.toi - skin_depth;
                    player_transform.translation += slide_velocity_direction * close_distance;

                    let time_delta = time_remainder;
                    let stop_time = close_distance / slide_velocity_magnitude;
                    let time_remainder = time_delta - stop_time;
                    let velocity_remainder = slide_velocity * time_remainder / time_delta;
                    let slide_velocity =
                        velocity_remainder.dot(second_slide_direction) * second_slide_direction;

                    if rapier_context
                        .cast_shape(
                            player_transform.translation,
                            Quat::IDENTITY,
                            slide_velocity.normalize(),
                            shape,
                            time_remainder * slide_velocity.length() + skin_depth,
                            QueryFilter::default(),
                        )
                        .is_none()
                    {
                        // Only move if there are no collisions only second slide axis, else only move up to second contact
                        player_transform.translation += slide_velocity * time_remainder;
                    }
                } else {
                    player_transform.translation += slide_velocity * time_remainder;
                }
            }
        } else {
            player_transform.translation += target_velocity * time_delta;
        }

        if !started_overlapping {
            rapier_context.intersections_with_shape(
                player_transform.translation + half_player_height * Vec3::Y,
                Quat::IDENTITY,
                shape,
                collision_filter,
                |_| {
                    // cast_shape sometimes lies about there being no collision due to float precision issues,
                    // so check for intersections and if found restore to starting position
                    warn!("Camera shape found to intersect world collider after movement, restoring to last valid position");
                    player_transform.translation = reset_position;
                    false
                }
            );
        }
    }
}

fn move_y(
    vertical_velocity: f32,
    player_transform: &mut Mut<Transform>,
    rapier_context: Res<RapierContext>,
    half_player_height: f32,
    direction: Vec3,
    shape: Collider,
    time_delta: f32,
    skin_depth: f32,
    collision_filter: QueryFilter,
    player: &mut Mut<Player>
) {
    if vertical_velocity.abs() > 0.0 {
        let reset_position = player_transform.translation;
        if let Some((_, hit)) = rapier_context.cast_shape(
            player_transform.translation + half_player_height * Vec3::Y,
            Quat::IDENTITY,
            direction,
            &shape,
            time_delta * vertical_velocity.abs() + skin_depth,
            collision_filter,
        ) {
            let close_distance = hit.toi - skin_depth;
            player_transform.translation += direction * close_distance;
            player.is_grounded = vertical_velocity < 0.0;
        } else {
            player_transform.translation += direction * vertical_velocity.abs() * time_delta;
            player.is_grounded = false;
        }

        rapier_context.intersections_with_shape(
            player_transform.translation + half_player_height * Vec3::Y,
            Quat::IDENTITY,
            &shape,
            collision_filter,
            |_| {
                // cast_shape sometimes lies about there being no collision due to float precision issues,
                // so check for intersections and if found restore to starting position 
                // NOTE: Have not seen this in the wild with pure vertical movement, yet
                warn!("Camera shape found to intersect world collider after vertical movement, restoring to last valid position");
                player_transform.translation = reset_position;
                false
            },
        );
    }
}

pub fn update_look(
    time: Res<Time>,
    player_input: Res<PlayerInput>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    mut player_query: Query<(&mut Transform, &Player)>,
) {
    let rotation_speed = 0.1; // TODO: degrees = dots * 0.022

    for (mut camera_transform, mut player_camera) in camera_query.iter_mut() {
        if let Ok((mut player_transform, player)) = player_query.get_mut(player_camera.target) {
            // prevent rotation past 10 degrees towards vertical
            let clamp_angle = std::f32::consts::PI * (0.5 - 10.0 / 180.0);

            let scaled_mouse_delta = rotation_speed * time.delta_seconds() * player_input.mouse_motion;

            let yaw = (player_camera.yaw - scaled_mouse_delta.x) % (2.0 * std::f32::consts::PI);
            let pitch = utils::clamp(
                player_camera.pitch - scaled_mouse_delta.y,
                -clamp_angle,
                clamp_angle,
            );

            player_transform.rotation = Quat::IDENTITY;
            player_transform.rotate_axis(Vec3::Y, yaw);

            camera_transform.rotation = Quat::IDENTITY;
            camera_transform.rotate_axis(Vec3::Y, yaw);
            let local_x = camera_transform.local_x();
            camera_transform.rotate_axis(local_x, pitch);

            if player.is_crouched {
                player_camera.offset.y = 0.75;
            } else {
                player_camera.offset.y = 1.25;
            }
            camera_transform.translation = player_transform.translation + player_camera.offset;

            player_camera.yaw = yaw;
            player_camera.pitch = pitch;
        }
    }
}
