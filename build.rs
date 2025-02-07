use std::{path::Path, process::Command};

fn main() {
    // Link against the GLFW static library
    println!("cargo:rustc-link-search=native=libs");
    println!("cargo:rustc-link-lib=static=libclang");
    println!("cargo:rustc-link-lib=static=glfw3");
    println!("cargo:rustc-link-lib=static=assimp");
    
    let resource_script_path = "./copy_files.sh";

    if !Path::new(resource_script_path).exists() {
        panic!("Script not found {}", resource_script_path);
    }

    let status = Command::new("bash")
        .arg(resource_script_path)
        .status()
        .expect("Failed to execute external Bash script");

    if !status.success() {
        panic!("Script failed with status: {:?}", status);
    }
}
