use std::{env, path::Path, process::Command};

fn main() {
    // Link against the GLFW static library
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-search=native=libs");
        println!("cargo:rustc-link-lib=static=libclang");
        println!("cargo:rustc-link-lib=static=glfw3");
        println!("cargo:rustc-link-lib=static=assimp");
    }
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=native=libs");
        println!("cargo:rustc-link-lib=dylib=clang");
        println!("cargo:rustc-link-lib=static=glfw3");
        println!("cargo:rustc-link-lib=dylib=assimp");
    }

    // Debug: Check if Bash is available
    #[cfg(target_os = "windows")]
    {
        let bash_status = Command::new("where")
            .arg("bash")
            .status()
            .expect("Failed to check for Bash installation");

        if !bash_status.success() {
            panic!("Bash not found! Ensure it's installed and in PATH.");
        }
    }

    //#[allow(clippy)]
    {
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
}
