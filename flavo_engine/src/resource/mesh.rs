use std::path::{Path, PathBuf};

use crate::resource::asset::Asset;

const NUM_MESH_LODS: u8 = 4;

struct MeshLod {
    
}

struct MeshNode {
    lods: [Option<MeshLod>; NUM_MESH_LODS as usize],
    // Parent index to global mesh node array in MeshAsset
    parent_idx: u16,
    // First child index to global mesh node array in MeshAsset
    child_idx: u16,
    // Next sibling index to global mesh node array in MeshAsset
    next_idx: u16
}

struct MeshAsset {
    asset_path: PathBuf
}

impl Asset for MeshAsset {
    fn load(asset_path: &Path) {

    }
}
