use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod atlas_loader;
mod input;
mod mesher;
mod player_camera;
mod scene_spawner;
mod utils;
mod voxel;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(VorldPlugin)
        .run();
}

pub struct VorldPlugin;

impl Plugin for VorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState {
            cursor_locked: false,
        });
        atlas_loader::init(app);
        voxel::init(app);
        input::insert_resources(app);

        app.add_startup_system(scene_spawner::spawn_lighting);

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
