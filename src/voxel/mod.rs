use super::atlas_loader::AtlasTexture;
use super::mesher;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_rapier3d::prelude::*;
use futures_lite::future;
use std::collections::HashMap;

pub mod block_ids;
pub mod chunk;
pub mod direction;
pub mod world;

pub mod prelude {
    pub use crate::voxel::block_ids::*;
    pub use crate::voxel::chunk::*;
    pub use crate::voxel::direction::*;
    pub use crate::voxel::world::*;
}
use prelude::*;

pub struct VoxelConfig {
    /// indexed on voxel id (0-255) and then direction (0-5) returns tile id (u32)
    /// NOTE: direction is from the perspective of the voxel, not the observer (i.e. forward not front or perhaps not "left as I look at it" if front is the forward direction)
    pub id_to_tile: [[u32; 6]; 256],
}

pub fn init(app: &mut App) {
    let mut look_up = [[0; 6]; 256];
    look_up[BlockIds::Grass as usize] = [1, 1, 0, 2, 1, 1];
    look_up[BlockIds::Soil as usize] = [2, 2, 2, 2, 2, 2];
    look_up[BlockIds::Stone as usize] = [3, 3, 3, 3, 3, 3];
    look_up[BlockIds::StoneSlab as usize] = [ 5, 5, 6, 6, 5, 5]; 
    look_up[BlockIds::StoneBlocks as usize] = [4, 4, 5, 5, 4, 4];
    look_up[BlockIds::Wood as usize] = [9, 9, 8, 8, 9, 9];
    look_up[BlockIds::Planks as usize] = [10, 10, 10, 10, 10, 10];
    look_up[BlockIds::Debug as usize] = [17, 18, 15, 16, 20, 19];
    look_up[BlockIds::Rink as usize] = [21, 21, 21, 21, 21, 21];
    app.insert_resource(VoxelConfig {
        id_to_tile: look_up,
    });
    app.add_startup_system(setup);
    app.add_system(handle_meshing_tasks);
}

#[derive(Component)]
struct ComputeChunkMeshes(Task<(IVec3, Vec<(u32, Mesh)>)>);

pub fn setup(mut commands: Commands, voxel_config: Res<VoxelConfig>) {
    let world = build_controller_test_vorld();
    async_instantiate_world(&mut commands, &world, &voxel_config)
}

#[allow(dead_code)]
fn build_chunk_test_vorld() -> Vorld {
    let mut world = Vorld {
        chunks: HashMap::new(),
    };

    for x in -16..32 {
        for z in -16..32 {
            world.add_voxel(BlockIds::Grass as u8, x, 0, z);
            if (point_in_chunk(x) == 0 || point_in_chunk(x) == 15)
                && (point_in_chunk(z) == 0 || point_in_chunk(z) == 15)
            {
                world.add_voxel(BlockIds::StoneBlocks as u8, x, 1, z);
            }
        }
    }

    // Palette / tile lookup debug
    // for i in 0..9 {
    //     world.add_voxel(i+1, 2 * i as i32 - 5, 2, 2);
    // }

    for x in 4..12 {
        for z in 4..12 {
            for y in 1..18 {
                if (x == 4 || x == 11 || z == 4 || z == 11)
                    && !((x == 7 || x == 8) && z == 4 && y <= 2)
                {
                    world.add_voxel(BlockIds::StoneBlocks as u8, x, y, z);
                }
            }
            world.add_voxel(BlockIds::StoneBlocks as u8, x, 18, z);
        }
    }

    world
}

#[allow(dead_code)]
fn build_controller_test_vorld() -> Vorld {
    let mut world = Vorld {
        chunks: HashMap::new(),
    };

    // Grass base!
    for x in -32..32 {
        for z in -32..32 {
            world.add_voxel(BlockIds::Grass as u8, x, -1, z);
        }
    }

    // Pyramid!
    for i in 0..4 {
        let hw = 4-i;
        for x in -hw..hw {
            for z in -hw..hw {
                world.add_voxel(BlockIds::StoneBlocks as u8, x, i, z);
            }
        }
    }

    // Jumps
    let x_offset = -16;
    let z_offset = -16;

    let mut z = 0;
    fill(&mut world, BlockIds::StoneBlocks as u8, 2, 1, 1, x_offset, 0, z + z_offset);

    z -= 1;
    for gap in 0..=5 {
        z += 2 + gap;
        fill(&mut world, BlockIds::StoneBlocks as u8, 2, 2, 2, x_offset, 0, z + z_offset);
    }
    
    world.add_voxel(BlockIds::StoneBlocks as u8, x_offset + 3, 0, z_offset + 5);
    fill(&mut world, BlockIds::StoneBlocks as u8, 1, 1, 2, x_offset + 3, 0, z_offset + 11);
    fill(&mut world, BlockIds::StoneBlocks as u8, 1, 1, 3, x_offset + 3, 0, z_offset + 22);

    fill(&mut world, BlockIds::StoneBlocks as u8, 2, 2, 2, x_offset + 4, 0, z_offset + 8);
    fill(&mut world, BlockIds::StoneBlocks as u8, 2, 2, 2, x_offset + 4, 0, z_offset + 16);

    // Grid
    let x_offset = 16;
    let z_offset = 0;
    
    for k in -1..=1 {
        for i in -1..=1 {
            world.add_voxel(BlockIds::StoneBlocks as u8, x_offset + 2 * i, 0, z_offset + 2 * k);
        }
    }

    // Arches
    let x_offset = -24;
    let z_offset = 0;

    for i in 0..2 {
        fill(&mut world, BlockIds::StoneBlocks as u8, 3, 3-i, 1, x_offset - 1, 0, z_offset - 4 * i);
        fill(&mut world, BlockIds::Air as u8, 1, 2-i, 1, x_offset, 0, z_offset - 4 * i);
    }

    // Larger arches!
    fill(&mut world, BlockIds::StoneBlocks as u8, 4, 3, 1, x_offset - 1, 0, z_offset + 4);
    fill(&mut world, BlockIds::Air as u8, 2, 2, 1, x_offset, 0, z_offset + 4);

    fill(&mut world, BlockIds::StoneBlocks as u8, 4, 4, 1, x_offset - 1, 0, z_offset + 8);
    fill(&mut world, BlockIds::Air as u8, 2, 3, 1, x_offset, 0, z_offset + 8);

    // Crouch jump test
    fill(&mut world, BlockIds::StoneBlocks as u8, 3, 3, 1, x_offset - 5, 0, z_offset);
    fill(&mut world, BlockIds::Air as u8, 1, 1, 1, x_offset - 4, 1, z_offset);

    world
}

fn fill(world: &mut Vorld, block: u8, width: i32, height: i32, depth: i32, x: i32 , y: i32, z: i32) {
    for j in 0..height {
        for k in 0..depth {
            for i in 0..width {
                world.add_voxel(block, x + i, y + j, z + k);
              }
        }
    }
}

fn point_in_chunk(v: i32) -> i32 {
    if v % CHUNK_SIZE_I32 == 0 {
        0
    } else if v >= 0 {
        v % CHUNK_SIZE_I32
    } else {
        v % CHUNK_SIZE_I32 + CHUNK_SIZE_I32
    }
}

pub fn async_instantiate_world(commands: &mut Commands, world: &Vorld, voxel_config: &VoxelConfig) {
    let mut chunk_slices = Vec::<VorldSlice>::new();
    for (key, _) in world.chunks.iter() {
        if let Some(slice) = world.get_slice_for_chunk(key) {
            chunk_slices.push(slice);
        }
    }

    let thread_pool = AsyncComputeTaskPool::get();
    let look_up = voxel_config.id_to_tile;

    while let Some(slice) = chunk_slices.pop() {
        let task = thread_pool.spawn(async move {
            (
                slice.chunk.indices,
                mesher::build_chunk_meshes(slice, look_up),
            )
        });
        commands.spawn().insert(ComputeChunkMeshes(task));
    }
}

fn handle_meshing_tasks(
    mut commands: Commands,
    mut meshing_tasks: Query<(Entity, &mut ComputeChunkMeshes)>,
    atlas: Res<AtlasTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut task) in meshing_tasks.iter_mut() {
        if let Some((key, mut tile_meshes)) = future::block_on(future::poll_once(&mut task.0)) {
            while let Some((tile_id, mesh)) = tile_meshes.pop() {
                let mut entity_commands = commands.spawn();
                if let Some(collider) =
                    Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh)
                {
                    entity_commands.insert(collider);
                } else {
                    error!("Unable to generate mesh collider");
                }
                entity_commands.insert_bundle(MaterialMeshBundle {
                    mesh: meshes.add(mesh),
                    material: atlas.materials[&tile_id].clone(),
                    transform: Transform::from_xyz(
                        key.x as f32 * CHUNK_SIZE_F32,
                        key.y as f32 * CHUNK_SIZE_F32,
                        key.z as f32 * CHUNK_SIZE_F32,
                    ),
                    ..default()
                });
            }
            commands.entity(entity).remove::<ComputeChunkMeshes>();
        }
    }
}
