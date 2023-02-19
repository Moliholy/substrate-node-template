use sp_io::hashing::sha2_256;
use sp_std::vec::{Vec};
use sp_std::vec;

// 1KB chunk size
const CHUNK_SIZE: usize = 1024;
const CHUNK_FILLER: [u8; 32] = [0u8; 32];

pub struct FileMerkleTree {
    pub file_bytes: Vec<u8>,
    pub merkle_tree: Vec<u8>,
}

impl FileMerkleTree {
    pub fn new(file_bytes: Vec<u8>) -> Self {
        let chunks = file_bytes.chunks(CHUNK_SIZE);
        let size = chunks.len();
        let mut tree = chunks.map(|chunk| {
            if chunk.len() != CHUNK_SIZE {
                // process last chunk
                let mut result = vec![0u8; CHUNK_SIZE];
                for (index, byte) in chunk.iter().enumerate() {
                    result[index] = *byte;
                }
                sha2_256(result.as_slice())
            } else {
                sha2_256(&chunk)
            }
        })
            .fold(Vec::<u8>::new(), |mut acc, hash| {
                acc.append(&mut hash.to_vec());
                acc
            });
        // make the tree a totally balanced binary tree
        let mut num_items = size.next_power_of_two();
        for _ in 0..(num_items - size) {
            tree.extend_from_slice(&CHUNK_FILLER);
        }
        let mut pos = 0;
        while num_items > 0 {
            for i in (pos..num_items).step_by(2) {
                let slice1 = &tree[i..(i + CHUNK_SIZE)];
                let slice2 = &tree[i + CHUNK_SIZE..(i + 2 * CHUNK_SIZE)];
                let mut result: Vec<u8> = Vec::with_capacity(CHUNK_SIZE * 2);
                result.extend_from_slice(slice1);
                result.extend_from_slice(slice2);
                let hash = sha2_256(&result.as_slice());
                tree.extend_from_slice(&hash);
            }
            pos = num_items;
            num_items /= 2;
        }
        Self {
            file_bytes,
            merkle_tree: tree,
        }
    }

    pub fn file_chunk_at(&self, position: usize) -> &[u8] {
        let count = self.piece_count();
        if position >= count {
            panic!("Invalid position: {}", position);
        }
        let pos = position * CHUNK_SIZE;
        let limit = if position == count - 1 {
            self.file_bytes.len()
        } else { pos + CHUNK_SIZE };
        &self.file_bytes[pos..limit]
    }

    pub fn piece_count(&self) -> usize {
        let len = self.file_bytes.len();
        if len % CHUNK_SIZE == 0 { len / CHUNK_SIZE } else { (len / CHUNK_SIZE) + 1 }
    }

    pub fn root_hash(&self) -> &[u8] {
        &self.merkle_tree[(self.merkle_tree.len() - CHUNK_SIZE)..self.merkle_tree.len()]
    }
}