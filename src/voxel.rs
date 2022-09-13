use super::atlas_loader::AtlasTexture;
use super::mesher;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Direction {
    Forward = 0,
    Back = 1,
    Up = 2,
    Down = 3,
    Left = 4,
    Right = 5,
}

pub struct VoxelConfig {
    // NOTE: direction is from the perspective of the voxel, not the observer (i.e. forward not front or perhaps not "left as I look at it" if front is the forward direction)
    pub id_to_tile: HashMap<u8, HashMap<Direction, u32>>,
}

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_I32: i32 = 16;
pub const CHUNK_SIZE_F32: f32 = 16.0;
pub const CHUNK_ARRAY_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub struct Vorld {
    pub chunks: HashMap<IVec3, Chunk>,
}

impl Vorld {
    /// gets chunk index for a voxel at index v in world space on a given axis
    fn get_chunk_index(v: i32) -> i32 {
        if v >= 0 || v % CHUNK_SIZE_I32 == 0 {
            v / CHUNK_SIZE_I32
        } else {
            (v / CHUNK_SIZE_I32) - 1
        }
    }

    fn get_chunk_key(x: i32, y: i32, z: i32) -> IVec3 {
        IVec3::new(Self::get_chunk_index(x), Self::get_chunk_index(y), Self::get_chunk_index(z))
    }

    fn get_block_indices(chunk_key: IVec3, x: i32, y: i32, z: i32) -> (usize, usize, usize) {
        (
            (x - chunk_key.x * CHUNK_SIZE_I32).try_into().unwrap(),
            (y - chunk_key.y * CHUNK_SIZE_I32).try_into().unwrap(),
            (z - chunk_key.z * CHUNK_SIZE_I32).try_into().unwrap()
        )
    }

    pub fn add_voxel(&mut self, id: u8, x: i32, y: i32, z: i32) {
        let key = Self::get_chunk_key(x, y, z);
        if let Some(chunk) = self.chunks.get_mut(&key) {
            let block_indicies = Self::get_block_indices(key, x, y, z);
            chunk.add_voxel(id, block_indicies.0, block_indicies.1, block_indicies.2);
        } else {
            let mut chunk = Chunk { indices: key, voxels: [ BlockIds::Air as u8; CHUNK_ARRAY_SIZE ]};
            let block_indicies = Self::get_block_indices(key, x, y, z);
            chunk.add_voxel(id, block_indicies.0, block_indicies.1, block_indicies.2);
            self.chunks.insert(key, chunk);
        }
    }

    pub fn get_voxel(&self, x: i32, y: i32, z: i32) -> u8 {
        let key = Self::get_chunk_key(x, y, z);
        if let Some(chunk) = self.chunks.get(&key) {
            let block_indicies = Self::get_block_indices(key, x, y, z);
            chunk.get_voxel(block_indicies.0, block_indicies.1, block_indicies.2)
        } else {
            BlockIds::Air as u8
        }
    }
}

#[test]
fn test_key() {
    assert_eq!(Vorld::get_chunk_index(-15), -1);
    assert_eq!(Vorld::get_chunk_index(-16), -1);
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

pub struct Chunk {
    pub indices: IVec3,
    pub voxels: [u8; CHUNK_ARRAY_SIZE],
}

impl Chunk {
    fn add_voxel(&mut self, id: u8, x: usize, y: usize, z: usize) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.voxels[x + CHUNK_SIZE * z + CHUNK_SIZE * CHUNK_SIZE * y] = id;
        } else {
            panic!("Received add_voxel instruction outside chunk bounds");
        }
    }
    fn get_voxel(&self, x: usize, y: usize, z: usize) -> u8 {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.voxels[x + CHUNK_SIZE * z + CHUNK_SIZE * CHUNK_SIZE * y]
        } else {
            panic!("Received get_voxel request outside chunk bounds");
        }
    }
}

#[repr(u8)]
#[allow(dead_code)]
enum BlockIds {
    Air = 0,
    Grass = 1,
    StoneBlocks = 2,
}

pub fn init(app: &mut App) {
    let mut look_up = HashMap::new();
    // GRASS
    look_up.insert(
        BlockIds::Grass as u8,
        HashMap::from([
            (Direction::Forward, 1),
            (Direction::Back, 1),
            (Direction::Up, 0),
            (Direction::Down, 2),
            (Direction::Left, 1),
            (Direction::Right, 1),
        ])
    );
    // STONE BLOCKS
    look_up.insert(
        BlockIds::StoneBlocks as u8,
        HashMap::from([
            (Direction::Forward, 4),
            (Direction::Back, 4),
            (Direction::Up, 5),
            (Direction::Down, 5),
            (Direction::Left, 4),
            (Direction::Right, 4),
        ])
    );

    app.insert_resource(VoxelConfig { id_to_tile: look_up });
    app.add_startup_system(setup);
}

pub fn setup(
    mut commands: Commands,
    atlas: Res<AtlasTexture>,
    voxel_config: Res<VoxelConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut vorld = Vorld {
        chunks: HashMap::new(),
    };

    for x in -16..32 {
        for z in -16..32 {
            vorld.add_voxel(BlockIds::Grass as u8, x, 0, z);
            if (point_in_chunk(x) == 0 || point_in_chunk(x) == 15) && (point_in_chunk(z) == 0 || point_in_chunk(z) == 15) {
                vorld.add_voxel(BlockIds::StoneBlocks as u8, x, 1, z);
            }
        }
    }

    for x in 4..=12 {
        for z in 4..=12 {
            for y in 1..18 {
                if (x == 4 || x == 12 || z == 4 || z == 12) && !((x == 8 || x == 9) && z == 4 && y <= 2) {
                    vorld.add_voxel(BlockIds::StoneBlocks as u8, x, y, z);
                }
            }
            vorld.add_voxel(BlockIds::StoneBlocks as u8, x, 18, z);
        }
    }

    for (key, chunk) in vorld.chunks.iter() {
        let mut tile_meshes = mesher::build_chunk_meshes(&chunk, &vorld, &voxel_config);

        while let Some((tile_id, mesh)) = tile_meshes.pop() {
            let mut entity_commands = commands.spawn();
            if let Some(collider) = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh) {
                entity_commands.insert(collider);
            } else {
                error!("Unable to generate mesh collider");
            }
            entity_commands.insert_bundle(MaterialMeshBundle {
                mesh: meshes.add(mesh),
                material: atlas.materials[&tile_id].clone(),
                transform: Transform::from_xyz(key.x as f32 * CHUNK_SIZE_F32, key.y as f32 * CHUNK_SIZE_F32, key.z as f32 * CHUNK_SIZE_F32),
                ..default()
            });
        }
    }
}
