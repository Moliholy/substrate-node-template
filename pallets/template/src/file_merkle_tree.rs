use codec::{Decode, Encode, EncodeLike};
use scale_info::{Path, Type, TypeInfo};
use scale_info::build::Fields;
use sp_io::hashing::sha2_256;
use sp_std::vec::Vec;
use sp_std::vec;

// 1KB chunk size
const CHUNK_SIZE: usize = 1024;
const CHUNK_FILLER: [u8; 32] = [0u8; 32];


#[derive(Default, Clone, PartialEq)]
pub struct FileMerkleTree {
    pub file_bytes: Vec<u8>,
    pub merkle_tree: Vec<u8>,
    pub pieces: u32,
}

impl Encode for FileMerkleTree {
    fn encode(&self) -> Vec<u8> {
        let file_size = (self.file_bytes.len() as u32).to_le_bytes();
        let mut result = Vec::from(file_size.as_slice());
        result.extend_from_slice(self.pieces.to_le_bytes().as_slice());
        result.extend_from_slice(&self.file_bytes.encode());
        result.extend_from_slice(&self.merkle_tree.encode());
        result
    }
}

impl Decode for FileMerkleTree {
    fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
        let mut buff = [0u8; 4];
        input.read(&mut buff)?;
        let file_size = u32::from_le_bytes(buff);
        input.read(&mut buff)?;
        let pieces = u32::from_le_bytes(buff);
        let mut file_bytes = vec![0u8; file_size as usize];
        input.read(&mut file_bytes)?;
        let mut merkle_tree = vec![0u8; file_size as usize];
        input.read(&mut merkle_tree)?;
        Ok(FileMerkleTree { file_bytes, merkle_tree, pieces })
    }
}

impl TypeInfo for FileMerkleTree {
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder().path(Path::new("FileMerkleTree", module_path!())).composite(
            Fields::named()
                .field(|f| {
                    f.ty::<Vec<u8>>()
                        .name("file_bytes")
                        .type_name("Vec<u8>")
                })
                .field(|f| {
                    f.ty::<Vec<u8>>()
                        .name("merkle_tree")
                        .type_name("Vec<u8>")
                })
                .field(|f| {
                    f.ty::<u32>()
                        .name("pieces")
                        .type_name("u32")
                })
        )
    }
}

impl EncodeLike for FileMerkleTree {}

impl FileMerkleTree {
    pub fn new(file_bytes: Vec<u8>) -> Self {
        let chunks = file_bytes.chunks(CHUNK_SIZE);
        let pieces = chunks.len();
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
        let mut num_items = pieces.next_power_of_two();
        for _ in 0..(num_items - pieces) {
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
            pieces: pieces as u32,
            merkle_tree: tree,
        }
    }

    pub fn file_chunk_at(&self, position: usize) -> &[u8] {
        let count = self.piece_count();
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

    pub fn merkle_root(&self) -> &[u8] {
        &self.merkle_tree[(self.merkle_tree.len() - CHUNK_SIZE)..self.merkle_tree.len()]
    }
}