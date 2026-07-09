use std::path::PathBuf;

pub const IMAGE_SIZE: u64 = 8 * 1024 * 1024 * 1024;

pub fn workspace_image(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/images")
        .join(name)
}

pub fn ensure_sparse_image(name: &str) -> PathBuf {
    let path = workspace_image("generated").join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    if path
        .metadata()
        .map(|meta| meta.len() >= IMAGE_SIZE)
        .unwrap_or(false)
    {
        return path;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .unwrap()
        .set_len(IMAGE_SIZE)
        .unwrap();
    path
}
