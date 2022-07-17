use std::path::Path;

pub trait Asset {
    fn load(asset_path: &Path);
}
