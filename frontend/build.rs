use std::process::Command;

#[cfg(target_os = "windows")]
const SHELL: &str = "cmd";
#[cfg(target_os = "windows")]
const FLAG: &str = "/C";

#[cfg(not(target_os = "windows"))]
const SHELL: &str = "sh";
#[cfg(not(target_os = "windows"))]
const FLAG: &str = "-c";

fn main() {
    println!("cargo:rerun-if-changed=tailwind.config.ts");

    // Create dist folder if missing
    std::fs::create_dir_all("dist").expect("Failed to create dist folder");

    let tailwind_cmd = "npx tailwindcss -c tailwind.config.ts -o output.css --minify";

    let status = Command::new(SHELL)
        .arg(FLAG)
        .arg(tailwind_cmd)
        .status()
        .expect("Failed to run Tailwind CLI through shell");

    if !status.success() {
        panic!("Tailwind CSS build failed");
    }
}
