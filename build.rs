use std::env;
use std::fs;
use std::path::Path;
fn main() {
    println!("cargo:rustc-link-search=native=lib");

    // Link against the GLFW static library
    println!("cargo:rustc-link-lib=static=glfw3");

    //#[cfg(target_os = "macos")]
    //{
    //    println!("cargo:rustc-link-lib=framework=Cocoa");
    //    println!("cargo:rustc-link-lib=framework=OpenGL");
    //    println!("cargo:rustc-link-lib=framework=IOKit");
    //}
}
