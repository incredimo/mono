use std::process::{Command, exit};
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use crate::utils::*;
// Remove reqwest and zip as they are no longer needed
// use std::io::{self, Write}; // Keep if other io ops need it, else remove self and Write if only ErrorKind is from io
// use reqwest::blocking::get;
// use zip::ZipArchive;

pub fn build_monodeamon(project_root: &Path, target_arch: &str) {
    log_message(&format!("Building for target arch: {}", target_arch));
    log_message("Checking for Android NDK...");

    let ndk_path = match find_ndk_path() {
        Ok(path) => path,
        Err(message) => {
            log_error(&message);
            exit(1);
        }
    };

    let host_tag = if cfg!(target_os = "windows") {
        "windows-x86_64"
    } else if cfg!(target_os = "linux") {
        "linux-x86_64"
    } else if cfg!(target_os = "macos") {
        "darwin-x86_64"
    } else {
        log_error("Unsupported host OS for NDK toolchain path detection.");
        exit(1);
    };

    let clang_path = ndk_path.join(format!("toolchains/llvm/prebuilt/{}/bin", host_tag));

    log_message(&format!("Using NDK at: {}", ndk_path.display()));
    log_message(&format!("Checking for linker in: {}", clang_path.display()));

    if !clang_path.exists() {
        log_error(&format!("Clang toolchain path does not exist: {}. Please check NDK installation and host_tag detection.", clang_path.display()));
        exit(1);
    }

    // List files in clang_path for debugging, can be removed later
    for entry in fs::read_dir(&clang_path).unwrap_or_else(|e| {
        log_error(&format!("Failed to read clang_path directory {}: {}", clang_path.display(), e));
        exit(1);
    }) {
        let entry = entry.unwrap_or_else(|e| {
            log_error(&format!("Failed to read entry in clang_path directory: {}", e));
            exit(1);
        });
        log_message(&format!("Found file: {}", entry.path().display()));
    }

    env::set_var("ANDROID_NDK_HOME", ndk_path.clone()); // Set for cargo build
    env::set_var("PATH", format!("{};{}", clang_path.display(), env::var("PATH").unwrap_or_default()));

    // Determine linker prefix based on target_arch
    let linker_prefix = match target_arch {
        "aarch64-linux-android" => "aarch64-linux-android",
        "armv7-linux-androideabi" => "armv7a-linux-androideabi", // Note: armv7a often used in NDK toolchain names
        "x86_64-linux-android" => "x86_64-linux-android",
        "i686-linux-android" => "i686-linux-android",
        _ => {
            log_error(&format!("Unsupported target architecture for linker: {}", target_arch));
            exit(1);
        }
    };

    let linker_name_base = format!("{}21-clang", linker_prefix); // API level 21, adjust if needed
    let linker_name = if cfg!(target_os = "windows") {
        format!("{}.cmd", linker_name_base)
    } else {
        linker_name_base
    };
    let linker_cmd = clang_path.join(&linker_name).to_str().unwrap_or_else(|| {
        log_error("Failed to convert linker path to string.");
        exit(1);
    }).replace("\\", "/"); // Linker command path, replacing backslashes for cross-platform compatibility.

    // Dynamically determine the AR (archiver) command name based on the target architecture's prefix.
    let ar_name = format!("{}-ar", linker_prefix);
    // Construct the full path to the AR command.
    let ar_path = clang_path.join(ar_name).to_str().unwrap_or_else(||{
        log_error("Failed to convert AR path to string.");
        exit(1);
    }).replace("\\", "/");


    let cargo_config = project_root.join(".cargo/config.toml");
    fs::create_dir_all(cargo_config.parent().unwrap()).expect("Failed to create .cargo directory");
    // Dynamically create the content for .cargo/config.toml to specify the
    // archiver (ar) and linker for the target architecture.
    let config_content = format!(
            r#"[target.{}]
ar = "{}"
linker = "{}"
"#,
            target_arch,
            ar_path,
            linker_cmd
        );
    fs::write(
        cargo_config,
        &config_content,
    ).expect("Failed to write config.toml");

    log_message("Building the monodeamon project...");
    let status = Command::new("cargo")
        .args(&["build", "--release", "--target", target_arch]) // Use target_arch
        .current_dir(project_root)
        .status()
        .expect("Failed to build monodeamon");

    if !status.success() {
        log_error("Failed to build monodeamon. Exiting.");
        exit(1);
    }

    log_message("monodeamon built successfully.");
}

pub fn find_ndk_path() -> Result<PathBuf, String> {
    if let Ok(ndk_home_val) = env::var("ANDROID_NDK_HOME") {
        let ndk_path = PathBuf::from(ndk_home_val);
        if ndk_path.exists() && ndk_path.is_dir() {
            log_message(&format!("Using ANDROID_NDK_HOME: {}", ndk_path.display()));
            // Check if this path itself contains version subdirectories
            if let Some(versioned_path) = find_highest_version(&ndk_path) {
                 log_message(&format!("Found highest version NDK at: {}", versioned_path.display()));
                return Ok(versioned_path);
            }
            // If not, assume the path is directly to the NDK root (e.g. .../ndk/25.2.9519653)
            return Ok(ndk_path);
        } else {
            log_error(&format!("ANDROID_NDK_HOME is set to '{}', but the directory does not exist or is not a directory.", ndk_path.display()));
        }
    }

    let home_dir = env::var("HOME").ok().map(PathBuf::from); // For Linux/macOS ~

    let mut possible_paths = Vec::new();

    if cfg!(target_os = "windows") {
        let username = env::var("USERNAME").unwrap_or_default();
        possible_paths.push(PathBuf::from(format!(r"C:\Users\{}\AppData\Local\Android\Sdk\ndk", username)));
        possible_paths.push(PathBuf::from(r"C:\Program Files\Android\Android Studio\ndk"));
        possible_paths.push(PathBuf::from(r"C:\Android\Sdk\ndk"));
    } else if cfg!(target_os = "linux") {
        possible_paths.push(PathBuf::from("/usr/lib/android-ndk"));
        possible_paths.push(PathBuf::from("/opt/android-ndk"));
        if let Some(ref home) = home_dir {
            possible_paths.push(home.join("Android/Sdk/ndk"));
            possible_paths.push(home.join(".android/sdk/ndk"));
        }
    } else if cfg!(target_os = "macos") {
        if let Some(ref home) = home_dir {
            possible_paths.push(home.join("Library/Android/sdk/ndk"));
        }
        possible_paths.push(PathBuf::from("/Applications/Android Studio.app/Contents/sdk/ndk"));
    }

    for base_path in possible_paths {
        if base_path.exists() && base_path.is_dir() {
            if let Some(versioned_path) = find_highest_version(&base_path) {
                log_message(&format!("Found highest version NDK at: {}", versioned_path.display()));
                return Ok(versioned_path);
            } else { // If no version subdirs, maybe base_path itself is an NDK root like "ndk/25.1.8937393"
                 // We need to check if this path "looks" like an NDK root.
                 // For example, check for presence of "toolchains" or "platforms" directory.
                if base_path.join("toolchains").exists() && base_path.join("platforms").exists() {
                    log_message(&format!("Found NDK at direct path: {}", base_path.display()));
                    return Ok(base_path);
                }
            }
        }
    }

    Err("Android NDK path not found. Please set the ANDROID_NDK_HOME environment variable or ensure NDK is installed in a common location.".to_string())
}

pub fn find_highest_version(ndk_base_path: &Path) -> Option<PathBuf> {
    let mut highest_version_path: Option<PathBuf> = None;
    let mut highest_version_name = String::new();

    if let Ok(entries) = fs::read_dir(ndk_base_path) {
        for entry_result in entries {
            if let Ok(entry) = entry_result {
                if entry.path().is_dir() {
                    let file_name = entry.file_name().into_string().unwrap_or_default();
                    // A simple check: NDK versions usually start with a number (e.g., "21.0.6113669", "25b")
                    // and don't typically contain "prebuilt" or "toolchains" at the top level of versioning.
                    if file_name.chars().next().map_or(false, |c| c.is_digit(10)) &&
                       !file_name.contains("prebuilt") && !file_name.contains("toolchains") {
                        // Attempt to parse version for comparison if needed, or rely on lexicographical sort.
                        // For simplicity, lexicographical sort of directory names often works for NDKs.
                        if highest_version_path.is_none() || file_name > highest_version_name {
                            // Check if this path itself "looks" like an NDK root.
                            let potential_ndk_root = entry.path();
                            if potential_ndk_root.join("toolchains").exists() && potential_ndk_root.join("platforms").exists() {
                                highest_version_name = file_name;
                                highest_version_path = Some(potential_ndk_root);
                            }
                        }
                    }
                }
            }
        }
    }
    if highest_version_path.is_some() {
         log_message(&format!("Selected NDK version {} from {}", highest_version_name, ndk_base_path.display()));
    } else {
        log_message(&format!("No suitable NDK version directory found in {}", ndk_base_path.display()));
    }
    highest_version_path
}
