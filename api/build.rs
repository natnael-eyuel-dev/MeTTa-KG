use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {

    // build the frontend if the directory exists
    let frontend_dir = PathBuf::from("../frontend");
    if frontend_dir.exists() {
        // run npm install and npm run build in the frontend directory
        let npm = if cfg!(target_os = "windows") { "npm.cmd" } else { "npm" };
        let _ = Command::new(npm)
            .args(["install"])
            .current_dir(&frontend_dir)
            .status();
        let status = Command::new(npm)
            .args(["run", "build"])
            .current_dir(&frontend_dir)
            .status()
            .expect("failed to run frontend build");
        if !status.success() {
            panic!("frontend build failed");
        }

        // copy the dist folder to ui-dist in the out dir
        let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let ui_dist = out_dir.join("ui-dist");
        let _ = fs::remove_dir_all(&ui_dist);
        fs::create_dir_all(&ui_dist).expect("failed to create ui-dist");

        let dist_dir = frontend_dir.join("dist");
        copy_dir_all(&dist_dir, &ui_dist).expect("failed to copy frontend dist to ui-dist");
    }
}

// recursively copy a directory
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
