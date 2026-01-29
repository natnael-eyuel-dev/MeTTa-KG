use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    eprintln!("[TRACE] Building MeTTa-KG with embedded assets...");

    let frontend_dir = PathBuf::from("../frontend");
    if frontend_dir.exists() {
        eprintln!(
            "[TRACE] Frontend directory found at: {}",
            frontend_dir.display()
        );
        let npm = "npm";
        let pnpm = "pnpm";

        if Command::new(pnpm).args(["--version"]).status().is_err() {
            eprintln!("[TRACE] pnpm not found, installing globally...");
            let install_pnpm_status = Command::new(npm)
                .args(["install", "-g", "pnpm"])
                .status()
                .expect("failed to run npm install -g pnpm");
            if !install_pnpm_status.success() {
                panic!("npm install -g pnpm failed");
            }
            eprintln!("[TRACE] pnpm installed successfully");
        } else {
            eprintln!("[TRACE] pnpm already available");
        }

        eprintln!("[TRACE] Installing frontend dependencies...");
        let install_status = Command::new(pnpm)
            .args(["install"])
            .current_dir(&frontend_dir)
            .status()
            .expect("failed to run npm install");
        if !install_status.success() {
            panic!("npm install failed");
        }
        eprintln!("[TRACE] Frontend dependencies installed");

        eprintln!("[TRACE] Building frontend...");
        let status = Command::new(pnpm)
            .args(["run", "build"])
            .current_dir(&frontend_dir)
            .status()
            .expect("failed to run frontend build");
        if !status.success() {
            panic!("frontend build failed");
        }
        eprintln!("[TRACE] Frontend build completed successfully");

        let out_dir: PathBuf = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let ui_dist = out_dir.join("ui-dist");
        eprintln!("[TRACE] Cleaning and preparing UI distribution directory...");
        let _ = fs::remove_dir_all(&ui_dist);
        fs::create_dir_all(&ui_dist).expect("failed to create ui-dist");

        let dist_dir = frontend_dir.join("dist");
        eprintln!("[TRACE] Copying frontend assets to ui-dist...");
        copy_dir_all(&dist_dir, &ui_dist).expect("failed to copy frontend dist to ui-dist");
        eprintln!("[TRACE] Frontend assets embedded successfully");
    } else {
        eprintln!("[TRACE] Frontend directory not found, skipping frontend build");
    }

    eprintln!("[TRACE] Setting up MORK server binary...");
    let mork_bin_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("mork-bin");
    let mork_binary_path = mork_bin_dir.join("mork-server");

    if mork_binary_path.exists() {
        eprintln!(
            "[TRACE] MORK binary found at: {}",
            mork_binary_path.display()
        );
    } else {
        eprintln!(
            "[TRACE] MORK binary not found at: {}",
            mork_binary_path.display()
        );
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if mork_binary_path.exists() {
            eprintln!("[TRACE] Setting executable permissions for MORK binary...");
            let mut perms = fs::metadata(&mork_binary_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&mork_binary_path, perms).unwrap();
            eprintln!("[TRACE] MORK binary permissions set");
        }
    }

    eprintln!("[TRACE] Build configuration complete");
    println!(
        "cargo:rustc-env=MORK_BINARY_PATH={}",
        mork_binary_path.display()
    );
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    let mut file_count = 0;
    let mut dir_count = 0;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            dir_count += 1;
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            file_count += 1;
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }

    if file_count > 0 || dir_count > 0 {
        eprintln!(
            "[TRACE] Copied {} files and {} directories from {} to {}",
            file_count,
            dir_count,
            src.display(),
            dst.display()
        );
    }

    Ok(())
}
