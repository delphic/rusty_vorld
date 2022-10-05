use bevy::prelude::*;
use bevy_hanabi::*;
use bevy_rapier3d::prelude::*;

mod gun;
mod health;
mod hit_flash;
mod player_input;
mod lifetime;
mod mesher;
mod named_collision_groups;
mod npc_spawner;
mod player;
mod projectile;
mod scene_spawner;
mod smoothed_follow;
mod utils;
mod voxel;
mod zombie;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(HanabiPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(VorldPlugin)
        .run();
}

pub struct VorldPlugin;

impl Plugin for VorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(GamePlugin);
        app.add_plugin(voxel::VoxelPlugin);
        app.add_plugin(player_input::PlayerInputPlugin);
        app.add_plugin(projectile::ProjectilePlugin);
        app.add_plugin(health::HealthPlugin);
        app.add_plugin(npc_spawner::NpcSpawnerPlugin);
        app.add_plugin(scene_spawner::SceneSpawnerPlugin);
        app.add_plugin(gun::GunPlugin);
        app.add_plugin(player::PlayerPlugin);
        app.add_plugin(hit_flash::HitFlashPlugin);
        app.add_plugin(zombie::NpcAiPlugin);
    }
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Core Systems
        app.insert_resource(GameState {
            cursor_locked: false,
        });
        app.add_system(grab_mouse);

        // Simple systems
        app.add_system(lifetime::update);
        app.add_system(smoothed_follow::follow.after(player::update_look));
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
