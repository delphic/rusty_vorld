use bevy::{input::mouse::MouseMotion, prelude::*};

pub struct PlayerInput {
    pub mouse_motion: Vec2,
    pub movement_direction: Vec3,
}

pub fn insert_resources(app: &mut App) {
    app.insert_resource(PlayerInput {
        mouse_motion: Vec2::ZERO,
        movement_direction: Vec3::ZERO,
    });
}

pub fn add_systems(app: &mut App) {
    app.add_system(detect_player_input);
}

fn detect_player_input(
    game_state: Res<crate::GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut player_input: ResMut<PlayerInput>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
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

    player_input.movement_direction = Vec3::new(delta_x, 0.0, delta_z);

    player_input.mouse_motion = Vec2::ZERO;
    if game_state.cursor_locked && !mouse_motion_events.is_empty() {
        for event in mouse_motion_events.iter() {
            player_input.mouse_motion += event.delta;
        }
    }
}
