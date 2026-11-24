//! Builds the vendored Blip package's C interface (`libBlipC.a`) with SwiftPM and links it.
//!
//! Blip is optional: on other platforms, or when Swift or the submodule is missing, the build
//! carries on without it, `cfg(blip)` is unset, and the overlay keeps drawing its own windows.

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(blip)");
    println!("cargo::rerun-if-changed=vendor/blip/Package.swift");
    println!("cargo::rerun-if-changed=vendor/blip/Sources");
    println!("cargo::rerun-if-env-changed=OPENLOGI_NO_BLIP");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos")
        || std::env::var_os("OPENLOGI_NO_BLIP").is_some()
    {
        return;
    }
    match build() {
        Ok(()) => println!("cargo::rustc-cfg=blip"),
        Err(reason) => println!("cargo::warning=Blip banners disabled: {reason}"),
    }
}

fn build() -> Result<(), String> {
    let package = Path::new(&std::env::var("CARGO_MANIFEST_DIR").map_err(|e| e.to_string())?)
        .join("vendor/blip");
    if !package.join("Package.swift").exists() {
        return Err("the vendor/blip submodule is empty (git submodule update --init)".into());
    }
    let scratch = PathBuf::from(std::env::var("OUT_DIR").map_err(|e| e.to_string())?).join("blip");
    let status = Command::new("swift")
        .args([
            "build",
            "-c",
            "release",
            "--product",
            "BlipC",
            "--package-path",
        ])
        .arg(&package)
        .arg("--scratch-path")
        .arg(&scratch)
        .status()
        .map_err(|e| format!("could not run swift: {e}"))?;
    if !status.success() {
        return Err(format!("swift build failed ({status})"));
    }
    println!(
        "cargo::rustc-link-search=native={}",
        scratch.join("release").display()
    );
    println!("cargo::rustc-link-lib=static=BlipC");
    // The Swift runtime's compatibility libraries, which the objects ask for at link time. The
    // runtime search path for execution is set by the binary's own build script.
    println!("cargo::rustc-link-search=native=/usr/lib/swift");
    if let Some(sdk) = tool_output("xcrun", &["--sdk", "macosx", "--show-sdk-path"]) {
        println!("cargo::rustc-link-search=native={sdk}/usr/lib/swift");
    }
    if let Some(swift) = tool_output("xcrun", &["--find", "swift"])
        && let Some(usr) = Path::new(&swift).parent().and_then(Path::parent)
    {
        println!(
            "cargo::rustc-link-search=native={}",
            usr.join("lib/swift/macosx").display()
        );
    }
    Ok(())
}

fn tool_output(tool: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(tool).args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
