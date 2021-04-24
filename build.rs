// Build script
// Copy resources folder
fn main() {
    println!("cargo:rerun-if-changed=resources");
    println!("cargo:rerun-if-changed=Rocket.toml");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.lock");
    // To log output use `eprintln!` and the `-vv` compiler flag
    // See: https://github.com/rust-lang/cargo/issues/985

    // Copy needed files for executable in order to work
    // Needed
    // * resources
    // * Rocket.toml

    //---
    // Getting the target folder...
    // Does not really work
    // let dst = PathBuf::from(std::env::var("CARGO_TARGET_DIR").unwrap());
    // Nor this
    // let dst = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    //---
    // Using this "hacky" solution for now
    let profile = std::env::var("PROFILE").unwrap();
    let dst_base = match profile.as_str() {
        "debug" => PathBuf::from("./target").join("debug"),
        "release" => PathBuf::from("./target").join("release"),
        _ => PathBuf::from("./target").join("debug"),
    };

    // Copy resources folder
    let src = PathBuf::from("./resources");
    let dst = dst_base.join("resources");
    let _ = copy_dir_all(src, dst);

    // Copy rocket config
    let src = PathBuf::from("./Rocket.toml");
    let dst = dst_base.join("Rocket.toml");
    let _ = fs::copy(src, dst);
}

use std::path::{Path, PathBuf};
use std::{fs, io};

// See: https://stackoverflow.com/a/65192210/12347616
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
