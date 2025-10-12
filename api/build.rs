use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let frontend_dir = PathBuf::from("../frontend");
    if frontend_dir.exists() {
        let npm = if cfg!(target_os = "windows") {
            "npm.cmd"
        } else {
            "npm"
        };
        let install_status = Command::new(npm)
            .args(["install"])
            .current_dir(&frontend_dir)
            .status()
            .expect("failed to run npm install");
        if !install_status.success() {
            panic!("npm install failed");
        }

        let status = Command::new(npm)
            .args(["run", "build"])
            .current_dir(&frontend_dir)
            .status()
            .expect("failed to run frontend build");
        if !status.success() {
            panic!("frontend build failed");
        }

        let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let ui_dist = out_dir.join("ui-dist");
        let _ = fs::remove_dir_all(&ui_dist);
        fs::create_dir_all(&ui_dist).expect("failed to create ui-dist");

        let dist_dir = frontend_dir.join("dist");
        copy_dir_all(&dist_dir, &ui_dist).expect("failed to copy frontend dist to ui-dist");
    }

    let mork_binary_path =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("mork-bin/mork-server");

    if mork_binary_path.exists() {
        println!("cargo:rerun-if-changed={}", mork_binary_path.display());
        println!(
            "cargo:rustc-env=MORK_BINARY_PATH={}",
            mork_binary_path.display()
        );
    } else {
        panic!(
            "Mork binary not found at '{}'. Make sure it exists in the mork-bin directory.",
            mork_binary_path.display()
        );
    }
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
