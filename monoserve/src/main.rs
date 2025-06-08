use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::fs::OpenOptions;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "monoserve")] // Optional: define a binary name for help messages
struct Cli {
    #[clap(short, long, default_value = "0.0.0.0", help = "Address to bind the server to")]
    bind_address: String,

    #[clap(short, long, default_value_t = 12345, help = "Port to listen on")]
    port: u16,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let listener_address = format!("{}:{}", cli.bind_address, cli.port);
    let listener = match TcpListener::bind(&listener_address) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", listener_address, e);
            return Err(e); // Or std::process::exit(1);
        }
    };

    println!("Server listening on {}", listener_address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_client(stream: TcpStream) {
    let client_addr = stream.peer_addr().unwrap();
    println!("New connection from {}", client_addr);

    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream for reader"));

    let mut id_line = String::new();
    let mut device_id_for_log_name; // Made mutable

    match reader.read_line(&mut id_line) {
        Ok(0) => {
            println!("Connection from {} closed before device ID was sent.", client_addr);
            return;
        }
        Ok(_) => {
            if id_line.starts_with("DEVID:") {
                // Sanitize the received device ID to create a safe filename.
                // Allows alphanumeric characters, hyphens, and underscores. Replaces others with '_'.
                device_id_for_log_name = id_line.trim_start_matches("DEVID:").trim().replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
                if device_id_for_log_name.is_empty() {
                    eprintln!("Warning: Received empty DEVID from {}. Using sanitized IP for log name.", client_addr);
                    // Sanitize IP address for filename as a fallback.
                    device_id_for_log_name = client_addr.to_string().replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
                }
            } else {
                eprintln!("Warning: First line from {} was not DEVID. Content: '{}'. Using sanitized IP for log name.", client_addr, id_line.trim());
                // Sanitize IP address for filename if DEVID is not provided or incorrect.
                device_id_for_log_name = client_addr.to_string().replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
            }
        }
        Err(e) => {
            eprintln!("Failed to read device ID line from {}: {}. Closing connection.", client_addr, e);
            return;
        }
    }

    // Open the log file for appending
    let log_file_name = format!("logs_{}.txt", device_id_for_log_name);
    let mut log_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_name) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to open log file {}: {}. Closing connection.", log_file_name, e);
                return;
            }
        };

    // If the first line was not DEVID, and we want to save it:
    if !id_line.starts_with("DEVID:") && !id_line.is_empty() {
        if let Err(e) = writeln!(log_file, "[NON-DEVID-PREFIX] {}", id_line.trim_end()) {
             eprintln!("Failed to write initial non-DEVID line to log: {}", e);
        }
    }

    // Write each line to the log file
    for line in reader.lines() {
        match line {
            Ok(log) => {
                if let Err(e) = writeln!(log_file, "{}", log) {
                    eprintln!("Failed to write to log file {}: {}", log_file_name, e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read line from {}: {}", client_addr, e);
                break;
            }
        }
    }

    println!("Connection from {} (Device ID: {}) closed", client_addr, device_id_for_log_name);
}
