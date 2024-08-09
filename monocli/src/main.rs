use std::process::{Command, exit};
use std::path::{Path, PathBuf};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: monocli <install|remove|check|dump>");
        exit(1);
    }

    let command = &args[1];
    let adb_path = "adb"; // Ensure adb is in your PATH or provide the full path
    let project_root = env::current_dir().unwrap().join("..").join("monodeamon"); // Path to monodeamon project
    let binary_path = project_root.join("target/aarch64-linux-android/release/monodeamon");

    match command.as_str() {
        "install" => {
            install_monodeamon(&adb_path, &project_root, &binary_path);
        },
        "remove" => {
            remove_monodeamon(&adb_path);
        },
        "check" => {
            check_device_status(&adb_path);
        },
        "dump" => {
            dump_device_data(&adb_path);
        },
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Usage: monocli <install|remove|check|dump>");
            exit(1);
        }
    }
}

fn dump_device_data(adb_path: &str) {
    let device_name = get_device_name(adb_path);
    let dump_dir = Path::new(&device_name);
    
    if !dump_dir.exists() {
        fs::create_dir_all(dump_dir).expect("Failed to create dump directory.");
    }

    // Capture logs
    capture_logs(adb_path, dump_dir);

    // Capture device information
    capture_device_info(adb_path, dump_dir);

    log_message(&format!("Data dumped to ./{}/", device_name));
}

fn get_device_name(adb_path: &str) -> String {
    let output = Command::new(adb_path)
        .arg("shell")
        .arg("getprop ro.product.model")
        .output()
        .expect("Failed to get device name");

    String::from_utf8_lossy(&output.stdout).trim().replace(" ", "_")
}

fn capture_logs(adb_path: &str, dump_dir: &Path) {
    let log_file = dump_dir.join("logs.txt");
    let mut log_output = File::create(&log_file).expect("Failed to create log file.");

    let output = Command::new(adb_path)
        .arg("logcat")
        .arg("-d")
        .output()
        .expect("Failed to capture logs");

    log_output
        .write_all(&output.stdout)
        .expect("Failed to write logs to file.");
}

fn capture_device_info(adb_path: &str, dump_dir: &Path) {
    let info_file = dump_dir.join("device_info.txt");
    let mut info_output = File::create(&info_file).expect("Failed to create device info file.");

    // Capture basic device information
    let commands = [
        "getprop",                    // System properties
        "dumpsys battery",            // Battery status
        "pm list packages",           // Installed packages
        "cat /proc/cpuinfo",          // CPU information
        "cat /proc/meminfo",          // Memory information
        "dumpsys window",             // Window information
        "dumpsys activity",           // Activity manager state
        "dumpsys power",              // Power manager state
    ];

    for cmd in &commands {
        let output = Command::new(adb_path)
            .arg("shell")
            .arg(cmd)
            .output()
            .expect(&format!("Failed to execute command: {}", cmd));

        writeln!(info_output, "### Output of {} ###\n", cmd).expect("Failed to write to device info file.");
        info_output
            .write_all(&output.stdout)
            .expect(&format!("Failed to write output of {} to file", cmd));
        writeln!(info_output, "\n\n").expect("Failed to write to device info file.");
    }
}


fn check_device_status(adb_path: &str) {
    // Check if the device is rooted
    let root_status = check_root_status(adb_path);
    if root_status {
        println!("[INFO] Device is rooted.");
    } else {
        println!("[INFO] Device is not rooted.");
    }

    // Check if monodeamon is running
    let monodeamon_status = check_monodeamon_status(adb_path);
    if monodeamon_status {
        println!("[INFO] monodeamon is running.");
    } else {
        println!("[INFO] monodeamon is not running.");
    }
}

fn check_root_status(adb_path: &str) -> bool {
    let output = Command::new(adb_path)
        .arg("shell")
        .arg("su -c id")
        .output()
        .expect("Failed to execute adb command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("uid=0(root)")
}

fn check_monodeamon_status(adb_path: &str) -> bool {
    let output = Command::new(adb_path)
        .arg("shell")
        .arg("ps | grep monodeamon")
        .output()
        .expect("Failed to execute adb command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    !stdout.is_empty()
}

fn install_monodeamon(adb_path: &str, project_root: &Path, binary_path: &Path) {
    // Check if monodeamon binary exists
    if !binary_path.exists() {
        println!("monodeamon binary not found. Building...");
        build_monodeamon(project_root);
    } else {
        println!("monodeamon binary found.");
    }

    // Push the binary to the device over USB
    Command::new(adb_path)
        .arg("push")
        .arg(binary_path.to_str().unwrap())
        .arg("/data/local/tmp/")
        .output()
        .expect("Failed to push the binary to the device");

    // Set executable permission
    Command::new(adb_path)
        .arg("shell")
        .arg("chmod")
        .arg("+x")
        .arg("/data/local/tmp/monodeamon")
        .output()
        .expect("Failed to set executable permissions");

    // Install the daemon as a service (requires root access)
    install_as_service(adb_path);

    // Start the daemon immediately
    Command::new(adb_path)
        .arg("shell")
        .arg("nohup /data/local/tmp/monodeamon &")
        .output()
        .expect("Failed to start the daemon");

    println!("Log collector daemon installed, set for persistence, and started successfully!");
}


use std::io::{self, Write};
use reqwest::blocking::get;
use zip::ZipArchive;
use std::fs::File;

 
fn build_monodeamon(project_root: &Path) {
    log_message("Checking for Android NDK...");

    // Attempt to automatically find the NDK path
    let ndk_path = find_ndk_path().expect("Failed to locate the Android NDK.");
    let clang_path = ndk_path.join("toolchains/llvm/prebuilt/windows-x86_64/bin");

    log_message(&format!("Found NDK at: {}", ndk_path.display()));
    log_message(&format!("Checking for linker in: {}", clang_path.display()));

    // List files in clang_path for debugging
    for entry in fs::read_dir(&clang_path).unwrap() {
        let entry = entry.unwrap();
        log_message(&format!("Found file: {}", entry.path().display()));
    }

    env::set_var("ANDROID_NDK_HOME", ndk_path.clone());
    env::set_var("PATH", format!("{};{}", clang_path.display(), env::var("PATH").unwrap()));

    // Choose a specific API level (e.g., 21) for the linker, using the .cmd wrapper
    let linker_name = "x86_64-linux-android21-clang.cmd";
    let linker_cmd = format!("{}/{}", clang_path.to_str().unwrap().replace("\\", "/"), linker_name);

    // Configure Rust to use the NDK toolchain with the specified linker
    let cargo_config = project_root.join(".cargo/config.toml");
    fs::create_dir_all(cargo_config.parent().unwrap()).expect("Failed to create .cargo directory");
    fs::write(
        cargo_config,
        &format!(
            r#"[target.x86_64-linux-android]
ar = "x86_64-linux-android-ar"
linker = "{}"
"#,
            linker_cmd
        ),
    ).expect("Failed to write config.toml");

    // Build the monodeamon project
    log_message("Building the monodeamon project...");
    let status = Command::new("cargo")
        .args(&["build", "--release", "--target", "x86_64-linux-android"])
        .current_dir(project_root)
        .status()
        .expect("Failed to build monodeamon");

    if !status.success() {
        log_error("Failed to build monodeamon. Exiting.");
        exit(1);
    }

    log_message("monodeamon built successfully.");
}

fn find_ndk_path() -> Option<PathBuf> {
    // Common installation paths for Android Studio on Windows
    let possible_paths = [
        r"C:\Users\%USERNAME%\AppData\Local\Android\Sdk\ndk",
        r"C:\Program Files\Android\Android Studio\ndk",
        r"C:\Android\Sdk\ndk",
        r"C:\Users\%USERNAME%\AppData\Local\Android\Sdk\ndk-bundle",
    ];

    for base_path in &possible_paths {
        let base_path = base_path.replace("%USERNAME%", &env::var("USERNAME").unwrap_or_default());
        let ndk_path = PathBuf::from(&base_path);

        if ndk_path.exists() {
            // Find the highest version of the NDK available
            if let Some(versioned_path) = find_highest_version(&ndk_path) {
                return Some(versioned_path);
            }
        }
    }

    None
}

fn find_highest_version(ndk_base_path: &Path) -> Option<PathBuf> {
    let mut highest_version = None;

    if let Ok(entries) = fs::read_dir(ndk_base_path) {
        for entry in entries.filter_map(Result::ok) {
            if entry.path().is_dir() {
                let file_name = entry.file_name().into_string().unwrap_or_default();
                if file_name.chars().next().unwrap_or('0').is_numeric() {
                    let version_path = entry.path();
                    highest_version = Some(version_path);
                }
            }
        }
    }

    highest_version
}

fn log_message(message: &str) {
    println!("[INFO] {}", message);
}

fn log_error(message: &str) {
    eprintln!("[ERROR] {}", message);
}

fn find_or_download_ndk() -> Option<PathBuf> {
    if let Some(ndk_path) = find_ndk_path() {
        return Some(ndk_path);
    }

    log_message("NDK not found. Attempting to download...");

    let download_url = "https://dl.google.com/android/repository/android-ndk-r21e-windows-x86_64.zip"; // Example URL, adjust as needed
    let ndk_download_path = download_ndk(download_url).expect("Failed to download the NDK.");

    log_message(&format!("Downloaded NDK to: {}", ndk_download_path.display()));

    let ndk_extract_path = extract_ndk(&ndk_download_path).expect("Failed to extract the NDK.");
    
    log_message(&format!("Extracted NDK to: {}", ndk_extract_path.display()));

    Some(ndk_extract_path)
}

 
fn download_ndk(url: &str) -> io::Result<PathBuf> {
    let output_path = env::temp_dir().join("android-ndk.zip");
    let mut file = File::create(&output_path)?;

    log_message("Starting NDK download...");
    let response = get(url).expect("Failed to download NDK.");
    let content = response.bytes().expect("Failed to read NDK download content.");
    file.write_all(&content)?;

    log_message("NDK downloaded successfully.");
    Ok(output_path)
}

fn extract_ndk(zip_path: &Path) -> io::Result<PathBuf> {
    let extract_path = env::temp_dir().join("android-ndk");

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    
    log_message("Extracting NDK...");
    archive.extract(&extract_path)?;

    // Assuming the NDK will be in a subdirectory like `android-ndk-r21e`
    for entry in fs::read_dir(&extract_path)? {
        let entry = entry?;
        if entry.path().is_dir() {
            return Ok(entry.path());
        }
    }

    Ok(extract_path)
}
 

fn install_as_service(adb_path: &str) {
    // Service script content
    let service_script = r#"
#!/system/bin/sh
while true; do
    /data/local/tmp/monodeamon
    sleep 10
done
"#;

    let service_script_path = "/data/local/tmp/monodeamon_service.sh";

    // Push the service script to the device
    let tmp_service_script = "/tmp/monodeamon_service.sh";
    fs::write(tmp_service_script, service_script).expect("Failed to write temporary service script");
    
    Command::new(adb_path)
        .arg("push")
        .arg(tmp_service_script)
        .arg(service_script_path)
        .output()
        .expect("Failed to push the service script to the device");

    // Set executable permission for the service script
    Command::new(adb_path)
        .arg("shell")
        .arg("chmod")
        .arg("+x")
        .arg(service_script_path)
        .output()
        .expect("Failed to set executable permissions for the service script");

    // Use Android's init system (if rooted) or set up the script to run at boot
    Command::new(adb_path)
        .arg("shell")
        .arg("su -c 'cp /data/local/tmp/monodeamon_service.sh /etc/init.d/'")
        .output()
        .expect("Failed to install the service script");

    println!("monodeamon service installed for persistence.");
}

fn remove_monodeamon(adb_path: &str) {
    // Stop the daemon if it's running
    Command::new(adb_path)
        .arg("shell")
        .arg("pkill -f /data/local/tmp/monodeamon")
        .output()
        .expect("Failed to stop the daemon");

    // Remove the daemon binary and service script
    Command::new(adb_path)
        .arg("shell")
        .arg("rm /data/local/tmp/monodeamon")
        .output()
        .expect("Failed to remove the daemon binary");

    Command::new(adb_path)
        .arg("shell")
        .arg("rm /data/local/tmp/monodeamon_service.sh")
        .output()
        .expect("Failed to remove the service script");

    // Remove the service from init.d (if rooted)
    Command::new(adb_path)
        .arg("shell")
        .arg("su -c 'rm /etc/init.d/monodeamon_service.sh'")
        .output()
        .expect("Failed to remove the service from init.d");

    println!("Log collector daemon removed successfully.");
}
