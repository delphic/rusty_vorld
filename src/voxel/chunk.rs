use bevy::prelude::IVec3;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_I32: i32 = 16;
pub const CHUNK_SIZE_F32: f32 = 16.0;
pub const CHUNK_ARRAY_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Copy, Clone, Debug)]
pub struct Chunk {
    pub indices: IVec3,
    pub voxels: [u8; CHUNK_ARRAY_SIZE],
}

impl Chunk {
    pub fn new(indices: IVec3, fill: u8) -> Self {
        Self {
            indices,
            voxels: [ fill; CHUNK_ARRAY_SIZE ],
        }
    }
    pub fn add_voxel(&mut self, id: u8, x: usize, y: usize, z: usize) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.voxels[x + CHUNK_SIZE * z + CHUNK_SIZE * CHUNK_SIZE * y] = id;
        } else {
            panic!("Received add_voxel instruction outside chunk bounds");
        }
    }
    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> u8 {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.voxels[x + CHUNK_SIZE * z + CHUNK_SIZE * CHUNK_SIZE * y]
        } else {
            panic!("Received get_voxel request outside chunk bounds");
        }
    }

    pub fn get_block_position(i: usize) -> (usize, usize, usize) {
        ( i % CHUNK_SIZE, i / (CHUNK_SIZE * CHUNK_SIZE), (i / CHUNK_SIZE) % CHUNK_SIZE)
    }
}