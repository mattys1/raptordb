use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") else {
        return;
    };

    let output = Command::new("git")
        .args(["config", "--local", "--get", "core.hooksPath"])
        .current_dir(&manifest_dir)
        .output();

    let Ok(output) = output else {
        return;
    };

    if output.status.success() {
        let current = String::from_utf8_lossy(&output.stdout);
        let current = current.trim();
        if current == ".githooks" {
            return;
        }
        if !current.is_empty() {
            println!("cargo:warning=core.hooksPath is {current}, leaving it unchanged");
            return;
        }
    }

    let status = Command::new("git")
        .args(["config", "--local", "core.hooksPath", ".githooks"])
        .current_dir(&manifest_dir)
        .status();

    if !matches!(status, Ok(status) if status.success()) {
        println!("cargo:warning=failed to set core.hooksPath to .githooks");
    }
}
