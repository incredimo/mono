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
 
 
use std::io::Write;
use chrono::Local;
use serde_json::json;
use regex::Regex;

// Add these to your Cargo.toml:
// [dependencies]
// chrono = "0.4"
// serde_json = "1.0"
// regex = "1.5"

fn dump_device_data(adb_path: &str) {
    let device_name = get_device_name(adb_path);
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let dump_dir = format!("./dump/{}_{}", device_name, timestamp);
    fs::create_dir_all(&dump_dir).expect("Failed to create dump directory.");

    let dump_dir = Path::new(&dump_dir);

    // Capture all information
    let device_info = capture_device_info(adb_path);
    let network_info = capture_network_info(adb_path);
    let storage_info = capture_storage_info(adb_path);
    let security_info = capture_security_info(adb_path);
    let system_settings = capture_system_settings(adb_path);
    let processes_and_services = capture_processes_and_services(adb_path);

    // Generate HTML report
    generate_html_report(
        dump_dir,
        &device_name,
        &timestamp,
        &device_info,
        &network_info,
        &storage_info,
        &security_info,
        &system_settings,
        &processes_and_services,
    );

    log_message(&format!("Advanced device dashboard generated in ./{}/", dump_dir.display()));
}

fn generate_html_report(
    dump_dir: &Path,
    device_name: &str,
    timestamp: &str,
    device_info: &str,
    network_info: &str,
    storage_info: &str,
    security_info: &str,
    system_settings: &str,
    processes_and_services: &str,
) {
    let mut html = String::new();

    // HTML structure
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str(&format!("<title>Device Dashboard - {} - {}</title>\n", device_name, timestamp));

    // CSS styles
    html.push_str("<style>\n");
    html.push_str(r#"
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; margin: 0; padding: 0; background-color: #f4f4f4; }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        h1, h2 { color: #2c3e50; }
        .dashboard { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; }
        .card { background-color: #fff; border-radius: 5px; box-shadow: 0 2px 5px rgba(0,0,0,0.1); padding: 20px; }
        .chart { width: 100%; height: 300px; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .collapsible { background-color: #777; color: white; cursor: pointer; padding: 18px; width: 100%; border: none; text-align: left; outline: none; font-size: 15px; }
        .active, .collapsible:hover { background-color: #555; }
        .content { padding: 0 18px; max-height: 0; overflow: hidden; transition: max-height 0.2s ease-out; background-color: #f1f1f1; }
        #searchInput { width: 100%; font-size: 16px; padding: 12px 20px 12px 40px; border: 1px solid #ddd; margin-bottom: 12px; }
    "#);
    html.push_str("</style>\n");

    // JavaScript libraries
    html.push_str("<script src=\"https://cdn.jsdelivr.net/npm/chart.js\"></script>\n");
    html.push_str("<script src=\"https://cdnjs.cloudflare.com/ajax/libs/moment.js/2.29.1/moment.min.js\"></script>\n");
    
    html.push_str("</head>\n<body>\n");

    // Dashboard content
    html.push_str("<div class=\"container\">\n");
    html.push_str(&format!("<h1>Device Dashboard - {}</h1>\n", device_name));
    html.push_str(&format!("<p>Generated on: {}</p>\n", timestamp));

    html.push_str("<div class=\"dashboard\">\n");
    
    // Quick stats
    add_quick_stats(&mut html, device_info, storage_info);

    // Charts
    add_chart(&mut html, "storageChart", "Storage Usage", parse_storage_info(storage_info));
    add_chart(&mut html, "batteryChart", "Battery Status", parse_battery_info(device_info));
    add_chart(&mut html, "memoryChart", "Memory Usage", parse_memory_info(device_info));
    add_chart(&mut html, "cpuChart", "CPU Usage", parse_cpu_info(processes_and_services));
    add_network_chart(&mut html, "networkChart", "Network Usage", parse_network_info(network_info));

    html.push_str("</div>\n"); // Close dashboard div

    // Collapsible sections
    html.push_str("<h2>Detailed Information</h2>\n");
    html.push_str("<input type=\"text\" id=\"searchInput\" onkeyup=\"searchTables()\" placeholder=\"Search for information...\">\n");
    
    add_collapsible_section(&mut html, "Device Information", device_info);
    add_collapsible_section(&mut html, "Network Information", network_info);
    add_collapsible_section(&mut html, "Storage Information", storage_info);
    add_collapsible_section(&mut html, "Security Information", security_info);
    add_collapsible_section(&mut html, "System Settings", system_settings);
    add_collapsible_section(&mut html, "Processes and Services", processes_and_services);

    html.push_str("</div>\n"); // Close container div

    // JavaScript for interactivity
    html.push_str("<script>\n");
    html.push_str(r#"
        var coll = document.getElementsByClassName("collapsible");
        var i;

        for (i = 0; i < coll.length; i++) {
            coll[i].addEventListener("click", function() {
                this.classList.toggle("active");
                var content = this.nextElementSibling;
                if (content.style.maxHeight){
                    content.style.maxHeight = null;
                } else {
                    content.style.maxHeight = content.scrollHeight + "px";
                } 
            });
        }

        function searchTables() {
            var input, filter, tables, tr, td, i, j, txtValue;
            input = document.getElementById("searchInput");
            filter = input.value.toUpperCase();
            tables = document.getElementsByTagName("table");
            for (i = 0; i < tables.length; i++) {
                tr = tables[i].getElementsByTagName("tr");
                for (j = 0; j < tr.length; j++) {
                    td = tr[j].getElementsByTagName("td");
                    for (var k = 0; k < td.length; k++) {
                        if (td[k]) {
                            txtValue = td[k].textContent || td[k].innerText;
                            if (txtValue.toUpperCase().indexOf(filter) > -1) {
                                tr[j].style.display = "";
                                break;
                            } else {
                                tr[j].style.display = "none";
                            }
                        }
                    }
                }
            }
        }
    "#);
    html.push_str("</script>\n");

    html.push_str("</body>\n</html>");

    let html_file = dump_dir.join("dashboard.html");
    let mut file = File::create(html_file).expect("Failed to create HTML file");
    file.write_all(html.as_bytes()).expect("Failed to write HTML content");
}

fn add_quick_stats(html: &mut String, device_info: &str, storage_info: &str) {
    html.push_str("<div class=\"card\">\n<h2>Quick Stats</h2>\n<ul>\n");
    
    let model = extract_info(device_info, r"ro.product.model\s*:\s*(.+)");
    let android_version = extract_info(device_info, r"ro.build.version.release\s*:\s*(.+)");
    let total_ram = extract_info(device_info, r"MemTotal:\s*(\d+)");
    let total_storage = extract_info(storage_info, r"/data\s+(\d+)");

    html.push_str(&format!("<li>Model: {}</li>\n", model));
    html.push_str(&format!("<li>Android Version: {}</li>\n", android_version));
    html.push_str(&format!("<li>Total RAM: {} KB</li>\n", total_ram));
    html.push_str(&format!("<li>Total Storage: {} KB</li>\n", total_storage));

    html.push_str("</ul>\n</div>\n");
}

fn add_chart(html: &mut String, chart_id: &str, title: &str, data: serde_json::Value) {
    html.push_str(&format!("<div class=\"card\">\n<h2>{}</h2>\n<div class=\"chart\"><canvas id=\"{}\"></canvas></div>\n</div>\n", title, chart_id));
    html.push_str("<script>\n");
    html.push_str(&format!(r#"
        new Chart(document.getElementById('{}').getContext('2d'), {{
            type: 'doughnut',
            data: {},
            options: {{
                responsive: true,
                plugins: {{
                    legend: {{
                        position: 'top',
                    }},
                    title: {{
                        display: true,
                        text: '{}'
                    }}
                }}
            }}
        }});
    "#, chart_id, data, title));
    html.push_str("</script>\n");
}

fn add_network_chart(html: &mut String, chart_id: &str, title: &str, data: serde_json::Value) {
    html.push_str(&format!("<div class=\"card\">\n<h2>{}</h2>\n<div class=\"chart\"><canvas id=\"{}\"></canvas></div>\n</div>\n", title, chart_id));
    html.push_str("<script>\n");
    html.push_str(&format!(r#"
        new Chart(document.getElementById('{}').getContext('2d'), {{
            type: 'line',
            data: {},
            options: {{
                responsive: true,
                plugins: {{
                    legend: {{
                        position: 'top',
                    }},
                    title: {{
                        display: true,
                        text: '{}'
                    }}
                }},
                scales: {{
                    x: {{
                        type: 'time',
                        time: {{
                            unit: 'minute'
                        }}
                    }},
                    y: {{
                        beginAtZero: true
                    }}
                }}
            }}
        }});
    "#, chart_id, data, title));
    html.push_str("</script>\n");
}

fn add_collapsible_section(html: &mut String, title: &str, content: &str) {
    html.push_str(&format!("<button class=\"collapsible\">{}</button>\n", title));
    html.push_str("<div class=\"content\">\n<table>\n");
    html.push_str("<tr><th>Property</th><th>Value</th></tr>\n");
    for line in content.lines() {
        if let Some((key, value)) = line.split_once(':') {
            html.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>\n", key.trim(), value.trim()));
        }
    }
    html.push_str("</table>\n</div>\n");
}

fn parse_storage_info(storage_info: &str) -> serde_json::Value {
    let mut total = 0;
    let mut used = 0;
    let df_regex = Regex::new(r"/data\s+(\d+)\s+(\d+)").unwrap();
    
    if let Some(captures) = df_regex.captures(storage_info) {
        total = captures[1].parse().unwrap_or(0);
        used = captures[2].parse().unwrap_or(0);
    }

    json!({
        "labels": ["Used", "Free"],
        "datasets": [{
            "data": [used, total - used],
            "backgroundColor": ["#FF6384", "#36A2EB"]
        }]
    })
}

fn parse_battery_info(device_info: &str) -> serde_json::Value {
    let mut level = 0;
    let battery_regex = Regex::new(r"level: (\d+)").unwrap();
    
    if let Some(captures) = battery_regex.captures(device_info) {
        level = captures[1].parse().unwrap_or(0);
    }

    json!({
        "labels": ["Battery", "Remaining"],
        "datasets": [{
            "data": [level, 100 - level],
            "backgroundColor": ["#FFCE56", "#E7E9ED"]
        }]
    })
}

fn parse_memory_info(device_info: &str) -> serde_json::Value {
    let mut total = 0;
    let mut free = 0;
    let mem_regex = Regex::new(r"MemTotal:\s+(\d+).*?MemFree:\s+(\d+)").unwrap();
    
    if let Some(captures) = mem_regex.captures(device_info) {
        total = captures[1].parse().unwrap_or(0);
        free = captures[2].parse().unwrap_or(0);
    }

    json!({
        "labels": ["Used", "Free"],
        "datasets": [{
            "data": [total - free, free],
            "backgroundColor": ["#4BC0C0", "#9966FF"]
        }]
    })
}

fn parse_cpu_info(processes_info: &str) -> serde_json::Value {
    let mut user = 0.0;
    let mut system = 0.0;
    let mut idle = 0.0;
    let cpu_regex = Regex::new(r"User (\d+)%, System (\d+)%, IOW (\d+)%, IRQ (\d+)%.*?(\d+)% TOTAL").unwrap();
    
    if let Some(captures) = cpu_regex.captures(processes_info) {
        user = captures[1].parse().unwrap_or(0.0);
        system = captures[2].parse().unwrap_or(0.0);
        idle = 100.0 - (captures[5].parse().unwrap_or(0.0));
    }

    json!({
        "labels": ["User", "System", "Idle"],
        "datasets": [{
            "data": [user, system, idle],
            "backgroundColor": ["#FF9F40", "#FF6384", "#4BC0C0"]
        }]
    })
}

fn parse_network_info(network_info: &str) -> serde_json::Value {
    let mut rx_bytes = Vec::new();
    let mut tx_bytes = Vec::new();
    let net_regex = Regex::new(r"(\d+) +(\d+)").unwrap();
    
    for (i, line) in network_info.lines().enumerate() {
        if let Some(captures) = net_regex.captures(line) {
            rx_bytes.push(json!({
                "x": format!("2023-01-01T00:{}:00", i.to_string().pad_left(2, '0')),
                "y": captures[1].parse::<u64>().unwrap_or(0)
            }));
            tx_bytes.push(json!({
                "x": format!("2023-01-01T00:{}:00", i.to_string().pad_left(2, '0')),
                "y": captures[2].parse::<u64>().unwrap_or(0)
            }));
        }
    }

    json!({
        "datasets": [
            {
                "label": "Received Bytes",
                "data": rx_bytes,
                "borderColor": "#36A2EB",
                "fill": false
            },
            {
                "label": "Transmitted Bytes",
                "data": tx_bytes,
                "borderColor": "#FF6384",
                "fill": false
            }
        ]
    })
}

fn capture_device_info(adb_path: &str) -> String {
    let commands = [
        "getprop",
        "dumpsys battery",
        "pm list packages -f",
        "cat /proc/cpuinfo",
        "cat /proc/meminfo",
        "dumpsys window",
        "dumpsys activity",
        "dumpsys power",
        "dumpsys bluetooth_manager",
        "dumpsys location",
        "dumpsys sensor_service",
        "dumpsys audio",
        "dumpsys camera",
        "dumpsys display",
    ];

    commands.iter().map(|cmd| execute_command(adb_path, cmd)).collect::<Vec<_>>().join("\n\n")
}

fn capture_network_info(adb_path: &str) -> String {
    let commands = [
        "ifconfig",
        "ip addr",
        "netstat -tuln",
        "dumpsys wifi",
        "dumpsys telephony.registry",
        "settings get global airplane_mode_on",
    ];

    commands.iter().map(|cmd| execute_command(adb_path, cmd)).collect::<Vec<_>>().join("\n\n")
}

fn capture_storage_info(adb_path: &str) -> String {
    let commands = [
        "df -h",
        "mount",
        "ls -lR /sdcard",
        "dumpsys mount",
    ];

    commands.iter().map(|cmd| execute_command(adb_path, cmd)).collect::<Vec<_>>().join("\n\n")
}

fn capture_security_info(adb_path: &str) -> String {
    let commands = [
        "getprop ro.boot.verifiedbootstate",
        "getprop ro.boot.flash.locked",
        "getprop ro.boot.vbmeta.device_state",
        "getprop ro.oem_unlock_supported",
        "settings get global development_settings_enabled",
        "pm list permissions -g -d",
    ];

    commands.iter().map(|cmd| execute_command(adb_path, cmd)).collect::<Vec<_>>().join("\n\n")
}

fn capture_system_settings(adb_path: &str) -> String {
    let commands = [
        "settings list global",
        "settings list system",
        "settings list secure",
    ];

    commands.iter().map(|cmd| execute_command(adb_path, cmd)).collect::<Vec<_>>().join("\n\n")
}

fn capture_processes_and_services(adb_path: &str) -> String {
    let commands = [
        "ps -ef",
        "top -n 1",
        "service list",
        "dumpsys activity services",
    ];

    commands.iter().map(|cmd| execute_command(adb_path, cmd)).collect::<Vec<_>>().join("\n\n")
}

fn execute_command(adb_path: &str, cmd: &str) -> String {
    let output = Command::new(adb_path)
        .arg("shell")
        .arg(cmd)
        .output()
        .expect(&format!("Failed to execute command: {}", cmd));

    format!("### Output of {} ###\n{}", cmd, String::from_utf8_lossy(&output.stdout))
}

fn get_device_name(adb_path: &str) -> String {
    let output = Command::new(adb_path)
        .arg("shell")
        .arg("getprop ro.product.model")
        .output()
        .expect("Failed to get device name");

    String::from_utf8_lossy(&output.stdout).trim().replace(" ", "_")
}

fn extract_info(info: &str, pattern: &str) -> String {
    let regex = Regex::new(pattern).unwrap();
    regex.captures(info)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

fn log_message(message: &str) {
    println!("[INFO] {}", message);
}

// Helper function to pad a string with leading zeros
trait PadLeft {
    fn pad_left(&self, width: usize, pad: char) -> String;
}

impl PadLeft for str {
    fn pad_left(&self, width: usize, pad: char) -> String {
        format!("{:>width$}", self, width = width).replace(' ', &pad.to_string())
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


use std::io::{self};
 
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

    unsafe {
        env::set_var("ANDROID_NDK_HOME", ndk_path.clone());
        env::set_var("PATH", format!("{};{}", clang_path.display(), env::var("PATH").unwrap()));
    }

 

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
