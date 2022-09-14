use bevy::prelude::IVec3;
use std::collections::HashMap;
use std::convert::TryInto;
use super::chunk::*;
use super::block_ids::*;

#[derive(Clone, Debug)]
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
        IVec3::new(
            Self::get_chunk_index(x),
            Self::get_chunk_index(y),
            Self::get_chunk_index(z),
        )
    }

    fn get_position_in_chunk(chunk_key: IVec3, x: i32, y: i32, z: i32) -> (usize, usize, usize) {
        (
            (x - chunk_key.x * CHUNK_SIZE_I32).try_into().unwrap(),
            (y - chunk_key.y * CHUNK_SIZE_I32).try_into().unwrap(),
            (z - chunk_key.z * CHUNK_SIZE_I32).try_into().unwrap(),
        )
    }

    pub fn add_voxel(&mut self, id: u8, x: i32, y: i32, z: i32) {
        let key = Self::get_chunk_key(x, y, z);
        if let Some(chunk) = self.chunks.get_mut(&key) {
            let block_indicies = Self::get_position_in_chunk(key, x, y, z);
            chunk.add_voxel(id, block_indicies.0, block_indicies.1, block_indicies.2);
        } else {
            let mut chunk = Chunk::new(key, BlockIds::Air as u8);
            let block_indicies = Self::get_position_in_chunk(key, x, y, z);
            chunk.add_voxel(id, block_indicies.0, block_indicies.1, block_indicies.2);
            self.chunks.insert(key, chunk);
        }
    }

    #[allow(dead_code)]
    pub fn get_voxel(&self, x: i32, y: i32, z: i32) -> u8 {
        let key = Self::get_chunk_key(x, y, z);
        if let Some(chunk) = self.chunks.get(&key) {
            let block_indicies = Self::get_position_in_chunk(key, x, y, z);
            chunk.get_voxel(block_indicies.0, block_indicies.1, block_indicies.2)
        } else {
            BlockIds::Air as u8
        }
    }

    pub fn get_slice_for_chunk(&self, chunk_key: &IVec3) -> Option<VorldSlice> {
        if let Some(chunk) = self.chunks.get(&chunk_key) {
            return Some(VorldSlice {
                chunk: *chunk,
                up_chunk: self.get_adjacent_chunk(chunk_key, IVec3::Y),
                down_chunk: self.get_adjacent_chunk(chunk_key, IVec3::NEG_Y),
                left_chunk: self.get_adjacent_chunk(chunk_key, IVec3::X),
                right_chunk: self.get_adjacent_chunk(chunk_key, IVec3::NEG_X),
                forward_chunk: self.get_adjacent_chunk(chunk_key, IVec3::Z),
                back_chunk: self.get_adjacent_chunk(chunk_key, IVec3::NEG_Z),
            });
        }
        None
    }

    fn get_adjacent_chunk(&self, chunk_key: &IVec3, offset: IVec3) -> Option<Chunk> {
        self.chunks.get(&IVec3::new(
            chunk_key.x + offset.x,
            chunk_key.y + offset.y,
            chunk_key.z + offset.z))
        .copied()
    }
}

/// Chunk and adjacent chunks data required for meshing
#[derive(Copy, Clone, Debug)]
pub struct VorldSlice {
    pub chunk: Chunk,
    pub up_chunk: Option<Chunk>,
    pub down_chunk: Option<Chunk>,
    pub left_chunk: Option<Chunk>,
    pub right_chunk: Option<Chunk>,
    pub forward_chunk: Option<Chunk>,
    pub back_chunk: Option<Chunk>,
}