use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::io::Write;
use std::time::Duration;
use std::thread;
use std::env; // Added use std::env

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <server_ip> <server_port>", args[0]);
        std::process::exit(1);
    }
    let server_ip = &args[1];
    let server_port = &args[2];
    let server_address = format!("{}:{}", server_ip, server_port);

    // Start a thread to capture and send logs
    thread::spawn(move || {
        loop {
            // Pass server_address.as_str() due to move, or clone server_address if it needs to be used elsewhere
            if let Err(e) = capture_and_send_logs(&server_address) {
                eprintln!("Error capturing logs: {}", e);
                thread::sleep(Duration::from_secs(5));
            }
        }
    });

    // Keep the main thread alive
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

// capture_and_send_logs function modified
fn capture_and_send_logs(server_address: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Get device ID
    // Note: Using absolute path /system/bin/getprop for robustness on device.
    let device_id_output = Command::new("/system/bin/getprop")
                                 .arg("ro.serialno")
                                 .output();
    let device_id = match device_id_output {
        Ok(output) if output.status.success() => {
            let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if id.is_empty() { "unknown_device".to_string() } else { id }
        }
        _ => "unknown_device".to_string(),
    };

    // Connect to the remote server
    let mut stream = TcpStream::connect(server_address)?;

    // Send device ID first
    if let Err(e) = stream.write_all(format!("DEVID:{}\n", device_id).as_bytes()) {
        eprintln!("Failed to send device ID: {}", e);
        // Optionally, return Err or handle to prevent further processing if ID send is critical
        // For now, we'll just print an error and continue trying to send logs.
    }

    // Start adb logcat command to capture logs
    // Note: If `logcat` command fails to spawn (e.g., not found, permissions), this will panic.
    // In a production scenario, this could be handled more gracefully, perhaps by attempting
    // to restart or logging a critical error before exiting the thread.
    let adb_process = Command::new("logcat")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = adb_process.stdout.ok_or("Failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    // Read logs and send to the server
    for line in reader.lines() {
        let line = line?;
        stream.write_all(line.as_bytes())?;
        stream.write_all(b"\n")?;
    }

    Ok(())
}
