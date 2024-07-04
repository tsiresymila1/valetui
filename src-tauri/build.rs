use std::{env, fs};
use std::path::PathBuf;
use fs_extra::dir::CopyOptions;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(out_dir).join("stubs");
    fs::create_dir_all(&dest_path).unwrap();
    fs_extra::dir::copy("src/stubs", &dest_path, &CopyOptions::new().overwrite(true)).unwrap();
    tauri_build::build()
}
