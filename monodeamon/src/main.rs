use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::io::Write;
use std::time::Duration;
use std::thread;

fn main() {
    let server_address = "192.168.1.100:12345"; // Replace with your server IP and port

    // Start a thread to capture and send logs
    thread::spawn(move || {
        loop {
            if let Err(e) = capture_and_send_logs(server_address) {
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

fn capture_and_send_logs(server_address: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Start adb logcat command to capture logs
    let adb_process = Command::new("logcat")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = adb_process.stdout.ok_or("Failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    // Connect to the remote server
    let mut stream = TcpStream::connect(server_address)?;

    // Read logs and send to the server
    for line in reader.lines() {
        let line = line?;
        stream.write_all(line.as_bytes())?;
        stream.write_all(b"\n")?;
    }

    Ok(())
}
