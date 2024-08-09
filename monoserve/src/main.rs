use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::fs::OpenOptions;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:12345")?; // Replace with your desired port

    println!("Server listening on port 12345");

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

    let reader = BufReader::new(stream);

    // Open the log file for appending
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("logs_{}.txt", client_addr))
        .unwrap();

    // Write each line to the log file
    for line in reader.lines() {
        match line {
            Ok(log) => {
                writeln!(log_file, "{}", log).unwrap();
            }
            Err(e) => {
                eprintln!("Failed to read line: {}", e);
                break;
            }
        }
    }

    println!("Connection from {} closed", client_addr);
}
