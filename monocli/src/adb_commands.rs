use std::process::Command; // exit removed
use std::path::Path; // PathBuf removed
use std::fs;
use crate::utils::*;
// crate::daemon_builder::build_monodeamon removed as it's called via qualified path
use crate::html_generator::generate_html_report;

pub fn check_device_status(adb_path: &str) {
    // Check if the device is rooted
    let root_status = check_root_status(adb_path);
    if root_status {
        log_message("Device is rooted.");
    } else {
        log_message("Device is not rooted.");
    }

    // Check if monodeamon is running
    let monodeamon_status = check_monodeamon_status(adb_path);
    if monodeamon_status {
        log_message("monodeamon is running.");
    } else {
        log_message("monodeamon is not running.");
    }
}

pub fn check_root_status(adb_path: &str) -> bool {
    let output = Command::new(adb_path)
        .arg("shell")
        .arg("su -c id")
        .output()
        .expect("Failed to execute adb command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("uid=0(root)")
}

pub fn check_monodeamon_status(adb_path: &str) -> bool {
    let output = Command::new(adb_path)
        .arg("shell")
        .arg("ps | grep monodeamon")
        .output()
        .expect("Failed to execute adb command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    !stdout.is_empty()
}

pub fn install_monodeamon(adb_path: &str, project_root: &Path, target_arch: &str, server_ip: &str, server_port: &str) {
    let binary_name = "monodeamon"; // Assuming the binary name is fixed
    let binary_path = project_root.join(format!("target/{}/release/{}", target_arch, binary_name));

    // Check if monodeamon binary exists
    if !binary_path.exists() {
        log_message(&format!("monodeamon binary not found for target {}. Building...", target_arch));
        // Pass target_arch to build_monodeamon
        crate::daemon_builder::build_monodeamon(&project_root, target_arch);
    } else {
        log_message(&format!("monodeamon binary found for target {}: {}", target_arch, binary_path.display()));
    }

    // Push the binary to the device over USB
    Command::new(adb_path)
        .arg("push")
        .arg(binary_path.to_str().unwrap_or_else(|| {
            // Handle cases where path conversion might fail, though unlikely for typical paths
            log_error(&format!("Failed to convert binary path to string: {}", binary_path.display()));
            std::process::exit(1); // or return an error
        }))
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
    let daemon_command = format!("nohup /data/local/tmp/monodeamon {} {} &", server_ip, server_port);
    Command::new(adb_path)
        .arg("shell")
        .arg(&daemon_command)
        .output()
        .expect("Failed to start the daemon");

    // Updated log message
    log_message("Log collector daemon installed and started successfully! Note: persistence details below.");
}

pub fn remove_monodeamon(adb_path: &str) {
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

    // The following command was for removing the service script, which is no longer created.
    // Command::new(adb_path)
        // .arg("shell")
        // .arg("rm /data/local/tmp/monodeamon_service.sh") // Removed
        // .output()
        // .expect("Failed to remove the service script");

    // Remove the service from init.d (if rooted)
    // Command::new(adb_path) // Removed
        // .arg("shell")
        // .arg("su -c 'rm /etc/init.d/monodeamon_service.sh'")
        // .output()
        // .expect("Failed to remove the service from init.d");

    log_message("Log collector daemon removed successfully.");
}

pub fn dump_device_data(adb_path: &str) {
    let device_name = get_device_name(adb_path);
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let dump_dir = format!("./dump/{}_{}", device_name, timestamp);
    fs::create_dir_all(&dump_dir).expect("Failed to create dump directory.");

    let dump_dir_path = Path::new(&dump_dir);

    // Capture all information
    let device_info = capture_device_info(adb_path);
    let network_info = capture_network_info(adb_path);
    let storage_info = capture_storage_info(adb_path);
    let security_info = capture_security_info(adb_path);
    let system_settings = capture_system_settings(adb_path);
    let processes_and_services = capture_processes_and_services(adb_path);

    // Generate HTML report
    generate_html_report(
        dump_dir_path,
        &device_name,
        &timestamp,
        &device_info,
        &network_info,
        &storage_info,
        &security_info,
        &system_settings,
        &processes_and_services,
    );

    log_message(&format!("Advanced device dashboard generated in ./{}/", dump_dir_path.display()));
}

pub fn capture_device_info(adb_path: &str) -> String {
    let commands: &[&str] = &[
        "getprop", // General properties
        "getprop ro.build.version.sdk", // SDK version
        "getprop ro.product.manufacturer", // Device manufacturer
        "getprop ro.product.name", // Device code name
        "dumpsys battery", // Battery status
        "pm list packages -f", // List all installed packages
        "cat /proc/cpuinfo", // CPU information
        "cat /proc/meminfo", // Memory information
        "dumpsys window", // Window manager state
        "dumpsys activity", // Activity manager state
        "dumpsys power", // Power manager state
        "dumpsys bluetooth_manager", // Bluetooth status
        "dumpsys location", // Location service status
        "dumpsys sensor_service", // Sensor service status
        "dumpsys audio", // Audio service status
        "dumpsys camera", // Camera service status (can be very verbose)
        "dumpsys display", // Display manager status
    ];

    commands.iter().map(|cmd| execute_command(adb_path, cmd)).collect::<Vec<_>>().join("\n\n")
}

pub fn capture_network_info(adb_path: &str) -> String {
    let commands: &[&str] = &[
        "ip addr show", // Preferred way to show IP addresses and network interfaces
        "ip route show", // IP routing table
        "getprop net.hostname", // Device hostname
        "netstat -tuln", // Active network connections (TCP, UDP, listening)
        "dumpsys wifi", // Wi-Fi service status
        "dumpsys telephony.registry", // Telephony information (SIM status, network type)
        "settings get global airplane_mode_on", // Airplane mode status
    ];

    commands.iter().map(|cmd| execute_command(adb_path, cmd)).collect::<Vec<_>>().join("\n\n")
}

pub fn capture_storage_info(adb_path: &str) -> String {
    let commands: &[&str] = &[
        "df -h", // Filesystem disk space usage (human-readable)
        "mount", // Mounted filesystems
        "ls -l /sdcard/", // List top-level of /sdcard (deeper scans should be optional)
        "dumpsys diskstats", // Disk I/O statistics
        // "dumpsys mount", // Removed as diskstats and mount might be sufficient
    ];

    commands.iter().map(|cmd| execute_command(adb_path, cmd)).collect::<Vec<_>>().join("\n\n")
}

pub fn capture_security_info(adb_path: &str) -> String {
    let commands: &[&str] = &[
        "getprop ro.boot.verifiedbootstate", // Verified boot state
        "getprop ro.boot.flash.locked", // Bootloader lock status
        "getprop ro.boot.vbmeta.device_state", // VBmeta device state
        "getprop ro.oem_unlock_supported", // OEM unlock support status
        "getprop ro.adb.secure", // ADB security status
        "getenforce", // SELinux status (enforcing/permissive/disabled)
        "settings get global development_settings_enabled", // Developer options status
        "pm list permissions -g -d", // List permissions (can be verbose)
    ];

    commands.iter().map(|cmd| execute_command(adb_path, cmd)).collect::<Vec<_>>().join("\n\n")
}

pub fn capture_system_settings(adb_path: &str) -> String {
    let commands = [
        "settings list global",
        "settings list system",
        "settings list secure",
    ];

    commands.iter().map(|cmd| execute_command(adb_path, cmd)).collect::<Vec<_>>().join("\n\n")
}

pub fn capture_processes_and_services(adb_path: &str) -> String {
    let commands = [
        "ps -A", // List all processes (broader compatibility)
        "top -n 1", // Top processes snapshot
        "service list", // List all system services
        "dumpsys activity services", // Detailed info on running services
        "dumpsys activity processes", // Detailed info on running processes
    ];

    commands.iter().map(|cmd| execute_command(adb_path, cmd)).collect::<Vec<_>>().join("\n\n")
}

pub fn execute_command(adb_path: &str, cmd: &str) -> String {
    let output = Command::new(adb_path)
        .arg("shell")
        .arg(cmd)
        .output()
        .expect(&format!("Failed to execute command: {}", cmd));

    format!("### Output of {} ###\n{}", cmd, String::from_utf8_lossy(&output.stdout))
}

pub fn get_device_name(adb_path: &str) -> String {
    let output = Command::new(adb_path)
        .arg("shell")
        .arg("getprop ro.product.model")
        .output()
        .expect("Failed to get device name");

    String::from_utf8_lossy(&output.stdout).trim().replace(" ", "_")
}

pub fn install_as_service(_adb_path: &str) { // _adb_path as it's no longer used directly
    log_message("Automated persistence script installation is currently not supported. `monodeamon` will run until the device reboots or the process is manually stopped. For persistent execution on rooted devices, manual setup of a boot script (e.g., via Magisk or init.rc) is required.");
}
