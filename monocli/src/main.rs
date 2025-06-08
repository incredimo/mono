use std::path::PathBuf;
use std::env;
// std::process::exit removed

// Modules
mod utils;
mod adb_commands;
mod daemon_builder;
mod html_generator;

// Use necessary functions from modules
// utils::log_error removed
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    Install {
        #[clap(long, default_value = "aarch64-linux-android")]
        target: String,
        #[clap(long, default_value = "127.0.0.1")]
        server_ip: String,
        #[clap(long, default_value = "12345")]
        server_port: String, // Keep as String for easier passing to ADB
    },
    Remove,
    Check,
    Dump,
}

fn main() {
    let cli = Cli::parse();

    let adb_path = "adb"; // Ensure adb is in your PATH or provide the full path

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Assuming monodeamon is in the parent directory of monocli's manifest dir (e.g. /app/monodeamon)
    let project_root_for_daemon = manifest_dir.parent().unwrap_or(&manifest_dir).join("monodeamon");

    match cli.command {
        Commands::Install { target, server_ip, server_port } => {
            // project_root_for_daemon is where the monodeamon project is located to be built
            adb_commands::install_monodeamon(&adb_path, &project_root_for_daemon, &target, &server_ip, &server_port);
        },
        Commands::Remove => {
            adb_commands::remove_monodeamon(&adb_path);
        },
        Commands::Check => {
            adb_commands::check_device_status(&adb_path);
        },
        Commands::Dump => {
            adb_commands::dump_device_data(&adb_path);
        },
    }
}
