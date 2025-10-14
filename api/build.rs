use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let frontend_dir = PathBuf::from("../frontend");
    if frontend_dir.exists() {
        let npm = "npm";
        let pnpm = "pnpm";

        if Command::new(pnpm).args(["--version"]).status().is_err() {
            let install_pnpm_status = Command::new(npm)
                .args(["install", "-g", "pnpm"])
                .status()
                .expect("failed to run npm install -g pnpm");
            if !install_pnpm_status.success() {
                panic!("npm install -g pnpm failed");
            }
        }

        let install_status = Command::new(pnpm)
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

    let mork_bin_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("mork-bin");
    let mork_binary_path = mork_bin_dir.join("mork-server");

    let url = "https://github.com/Qoba-ai/MeTTa-KG/releases/download/stable/mork_server-x86_64-unknown-linux-gnu";
    println!("Mork binary missing - downloading from {url}");

    fs::create_dir_all(&mork_bin_dir).expect("failed to create mork-bin directory");

    let status = Command::new("curl")
        .args(["-L", "-o"])
        .arg(&mork_binary_path)
        .arg(url)
        .status()
        .expect("failed to run curl");

    if !status.success() {
        panic!("Failed to download Mork binary from {url}");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&mork_binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mork_binary_path, perms).unwrap();
    }

    println!(
        "cargo:rustc-env=MORK_BINARY_PATH={}",
        mork_binary_path.display()
    );
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
