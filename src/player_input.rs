use bevy::{input::mouse::MouseMotion, prelude::*};

pub struct PlayerInput {
    pub mouse_motion: Vec2,
    pub movement_direction: Vec3,
    pub jump_requested: bool,
    pub crouch_requested: bool,
    pub shoot_requested: bool,
}

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerInput {
            mouse_motion: Vec2::ZERO,
            movement_direction: Vec3::ZERO,
            jump_requested: false,
            crouch_requested: false,
            shoot_requested: false,
        });
        app.add_system(detect_player_input);
    }
}

fn detect_player_input(
    game_state: Res<crate::GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
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

    if delta_x != 0.0 && delta_z != 0.0 {
        delta_x /= std::f32::consts::SQRT_2;
        delta_z /= std::f32::consts::SQRT_2;
    }

    player_input.movement_direction = Vec3::new(delta_x, 0.0, delta_z);

    player_input.mouse_motion = Vec2::ZERO;
    if game_state.cursor_locked && !mouse_motion_events.is_empty() {
        for event in mouse_motion_events.iter() {
            player_input.mouse_motion += event.delta;
        }
    }

    player_input.jump_requested = player_input.jump_requested || keyboard_input.just_pressed(KeyCode::Space);
    player_input.crouch_requested = keyboard_input.pressed(KeyCode::LControl);
    player_input.shoot_requested = player_input.shoot_requested || mouse_button_input.just_pressed(MouseButton::Left);
}
