use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf}; // Added PathBuf
use serde_json::json;
use regex::Regex;
// crate::utils::* removed (unused import)
// chrono::Local removed as timestamp is passed in

use tera::{Tera, Context};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera = Tera::default();
        let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
        let template_file_path = templates_dir.join("dashboard.html.tera");

        match tera.add_template_file(&template_file_path, Some("dashboard.html.tera")) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("FATAL: Failed to load dashboard template: {}", e);
                eprintln!("Template path attempted: {:?}", template_file_path);
                eprintln!("Current dir: {}", std::env::current_dir().unwrap().display());
                eprintln!("CARGO_MANIFEST_DIR: {}", env!("CARGO_MANIFEST_DIR"));
                if templates_dir.exists() && templates_dir.is_dir() {
                    eprintln!("Templates directory content:");
                    match std::fs::read_dir(&templates_dir) { // Explicitly match on the Result of read_dir
                        Ok(entries) => {
                            for entry_result in entries { // entry_result is Result<DirEntry, io::Error>
                                match entry_result {
                                    Ok(entry) => eprintln!("  Found: {:?}", entry.path()),
                                    Err(e_fs) => eprintln!("  Error reading an entry: {}", e_fs),
                                }
                            }
                        }
                        Err(e_rd) => eprintln!("  Error reading templates directory itself: {}", e_rd),
                    }
                } else {
                     eprintln!("Templates directory does not exist or is not a directory: {}", templates_dir.display());
                }
                std::process::exit(1);
            }
        }
        tera
    };
}

pub fn generate_html_report(
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
    let mut context = Context::new();
    context.insert("device_name", device_name);
    context.insert("timestamp", timestamp);

    let quick_stats_data = json!({
        "model": extract_info(device_info, r"ro.product.model\s*:\s*(.+)"),
        "android_version": extract_info(device_info, r"ro.build.version.release\s*:\s*(.+)"),
        "total_ram": extract_info(device_info, r"MemTotal:\s*(\d+)"),
        "total_storage": extract_info(storage_info, r"/data\s+(\d+)")
    });
    context.insert("quick_stats", &quick_stats_data);

    context.insert("storage_chart_data", &parse_storage_info(storage_info).to_string());
    context.insert("battery_chart_data", &parse_battery_info(device_info).to_string());
    context.insert("memory_chart_data", &parse_memory_info(device_info).to_string());
    context.insert("cpu_chart_data", &parse_cpu_info(processes_and_services).to_string());
    context.insert("network_chart_data", &parse_network_info(network_info).to_string());

    fn to_key_value_vec(s: &str) -> Vec<serde_json::Value> {
        s.lines()
         .filter_map(|line| {
             let parts: Vec<&str> = line.splitn(2, ':').collect();
             if parts.len() == 2 {
                 Some(json!({"key": parts[0].trim(), "value": parts[1].trim()}))
             } else if !line.trim().is_empty() && !line.starts_with("###") { // Handle lines without ':' but not section headers
                 Some(json!({"key": line.trim(), "value": ""}))
             } else {
                 None
             }
         })
         .collect()
    }

    let detailed_sections_data = vec![
        json!({"title": "Device Information", "data": to_key_value_vec(device_info)}),
        json!({"title": "Network Information", "data": to_key_value_vec(network_info)}),
        json!({"title": "Storage Information", "data": to_key_value_vec(storage_info)}),
        json!({"title": "Security Information", "data": to_key_value_vec(security_info)}),
        json!({"title": "System Settings", "data": to_key_value_vec(system_settings)}),
        json!({"title": "Processes and Services", "data": to_key_value_vec(processes_and_services)}),
    ];
    context.insert("detailed_sections", &detailed_sections_data);

    let html_content = TERA.render("dashboard.html.tera", &context)
                        .unwrap_or_else(|e| {
                            eprintln!("Tera template rendering error: {}", e);
                            format!("Failed to render template: {}", e) // Basic fallback
                        });

    let html_file = dump_dir.join("dashboard.html");
    let mut file = File::create(html_file).expect("Failed to create HTML file");
    file.write_all(html_content.as_bytes()).expect("Failed to write HTML content");
}

// parse_*_info functions remain as they are used to generate data for the context
pub fn parse_storage_info(storage_info: &str) -> serde_json::Value {
    let mut total = 0;
    let mut used = 0;
    let df_regex = Regex::new(r"/data\s+(\d+)\s+(\d+)").unwrap(); // Assuming KB for simplicity

    if let Some(captures) = df_regex.captures(storage_info) {
        total = captures[1].parse().unwrap_or(0);
        used = captures[2].parse().unwrap_or(0);
    }

    json!({
        "labels": ["Used Storage (KB)", "Free Storage (KB)"],
        "datasets": [{
            "data": [used, if total > used { total - used } else { 0 }],
            "backgroundColor": ["#FF6384", "#36A2EB"]
        }]
    })
}

pub fn parse_battery_info(device_info: &str) -> serde_json::Value {
    let mut level = 0;
    // More robust regex for battery level
    let battery_regex = Regex::new(r"(?i)level: (\d+)").unwrap();

    if let Some(captures) = battery_regex.captures(device_info) {
        level = captures[1].parse().unwrap_or(0);
    }

    json!({
        "labels": ["Battery Level", "Remaining Capacity"],
        "datasets": [{
            "data": [level, 100 - level],
            "backgroundColor": ["#FFCE56", "#E7E9ED"]
        }]
    })
}

pub fn parse_memory_info(device_info: &str) -> serde_json::Value {
    let mut total = 0;
    let mut free = 0;
    // Regex for MemTotal and MemFree, looking for lines starting with these
    let mem_total_regex = Regex::new(r"MemTotal:\s*(\d+)\s*kB").unwrap();
    let mem_free_regex = Regex::new(r"MemFree:\s*(\d+)\s*kB").unwrap();

    if let Some(captures) = mem_total_regex.captures(device_info) {
        total = captures[1].parse().unwrap_or(0);
    }
    if let Some(captures) = mem_free_regex.captures(device_info) {
        free = captures[1].parse().unwrap_or(0);
    }

    json!({
        "labels": ["Used Memory (KB)", "Free Memory (KB)"],
        "datasets": [{
            "data": [if total > free { total - free } else { 0 }, free],
            "backgroundColor": ["#4BC0C0", "#9966FF"]
        }]
    })
}

pub fn parse_cpu_info(processes_info: &str) -> serde_json::Value {
    let mut user = 0.0;
    let mut system = 0.0;
    let mut idle = 100.0; // Default to 100% idle if not found

    // Regex for CPU usage from 'top -n 1' (example format, might vary by Android version/busybox)
    // Example: "CPU:  1% usr  2% sys  0% nic 95% idle  0% io  0% irq  0% sirq"
    // Or "User 5%, System 10%, IOW 0%, IRQ 0%" (from 'top' output in initial main.rs)
    let cpu_regex_top = Regex::new(r"(\d+)% usr.*?(\d+)% sys.*?(\d+)% idle").or_else(|_|
                          Regex::new(r"User (\d+)%, System (\d+)%.*?(\d+)% TOTAL"));

    if let Ok(regex) = cpu_regex_top {
        if let Some(captures) = regex.captures(processes_info) {
            if captures.len() == 4 { // For "usr, sys, idle" format
                user = captures[1].parse().unwrap_or(0.0);
                system = captures[2].parse().unwrap_or(0.0);
                idle = captures[3].parse().unwrap_or(100.0);
            } else if captures.len() == 3 { // For "User X%, System Y%, Z% TOTAL"
                 user = captures[1].parse().unwrap_or(0.0);
                 system = captures[2].parse().unwrap_or(0.0);
                 let total_used: f64 = processes_info.lines()
                    .find(|line| line.contains("TOTAL"))
                    .and_then(|line| Regex::new(r"(\d+)% TOTAL").unwrap().captures(line))
                    .and_then(|cap| cap[1].parse().ok())
                    .unwrap_or(user + system); // Fallback if TOTAL not found
                idle = 100.0 - total_used;
                if idle < 0.0 { idle = 0.0;}

            }
        }
    }

    json!({
        "labels": ["User CPU", "System CPU", "Idle CPU"],
        "datasets": [{
            "data": [user, system, idle],
            "backgroundColor": ["#FF9F40", "#FF6384", "#4BC0C0"]
        }]
    })
}


pub fn parse_network_info(network_info: &str) -> serde_json::Value {
    let mut rx_bytes_total: u64 = 0;
    let mut tx_bytes_total: u64 = 0;

    // Try to parse 'ip -s link' or 'ifconfig' for summary stats if available
    // This is a simplified example; real parsing would be more complex.
    // Example: looking for lines like `RX bytes:12345... TX bytes:67890...` in ifconfig output
    // Or lines from `ip -s link`
    let rx_regex = Regex::new(r"RX packets \d+  bytes (\d+)").unwrap();
    let tx_regex = Regex::new(r"TX packets \d+  bytes (\d+)").unwrap();

    for line in network_info.lines() {
        if let Some(cap) = rx_regex.captures(line) {
            rx_bytes_total += cap[1].parse::<u64>().unwrap_or(0);
        }
        if let Some(cap) = tx_regex.captures(line) {
            tx_bytes_total += cap[1].parse::<u64>().unwrap_or(0);
        }
    }
    // For the line chart, we'd ideally have time series data.
    // Since we only have snapshots from commands, this will be a point-in-time total.
    // The template expects time series, so we'll provide a single point.
    // A better approach would be to collect data over time if monodeamon could do that.
    let now = chrono::Local::now().to_rfc3339(); // Using chrono for timestamp
    json!({
        "datasets": [
            {
                "label": "Total Received Bytes",
                "data": [{"x": now, "y": rx_bytes_total}],
                "borderColor": "#36A2EB",
                "fill": false
            },
            {
                "label": "Total Transmitted Bytes",
                "data": [{"x": now, "y": tx_bytes_total}],
                "borderColor": "#FF6384",
                "fill": false
            }
        ]
    })
}


// extract_info function remains useful
pub fn extract_info(info: &str, pattern: &str) -> String {
    let regex = Regex::new(pattern).unwrap();
    regex.captures(info)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "N/A".to_string())
}
