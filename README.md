# mono

Mono is a Rust-based automated device data extraction and persistent remote logging mechanism designed for Android devices. This project provides a toolset for managing a background daemon (`monodeamon`) that captures logs and sends them to a remote server (`monoserve`). `monocli` is used to build and deploy `monodeamon` to the Android device, configuring it with the server details.

## Table of Contents

- [mono](#mono)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Features](#features)
  - [Installation](#installation)
    - [Prerequisites](#prerequisites)
    - [Cloning the Repository](#cloning-the-repository)
  - [Usage](#usage)
    - [Building](#building)
    - [Running `monocli`](#running-monocli)
    - [Running `monoserve`](#running-monoserve)
    - [Commands (`monocli`)](#commands-monocli)
    - [Examples](#examples)
  - [Device Information Captured](#device-information-captured)
  - [HTML Report](#html-report)
  - [Persistence](#persistence)
  - [Experimental Directory](#experimental-directory)
  - [Contributing](#contributing)
    - [Setting Up for Development](#setting-up-for-development)

## Overview

Mono consists of three main components:

1.  **`monocli`**: A command-line interface (CLI) for cross-compiling and deploying the `monodeamon` to Android devices via ADB. It also handles removing the daemon, checking device status, and dumping comprehensive device information. `monodeamon` is configured by `monocli` during deployment.
2.  **`monodeamon`**: A Rust-based daemon that runs on Android devices. It captures logs (`logcat`) and sends them, along with a device identifier, to a configured `monoserve` instance. It receives the server IP and port as command-line arguments from `monocli` during startup.
3.  **`monoserve`**: A Rust-based TCP server that listens for connections from `monodeamon` instances. It receives logs and saves them to files named `logs_<device_id>.txt` (or `logs_<sanitized_ip>.txt` as a fallback).

## Features

- **Automated Data Extraction**: `monocli dump` automates capturing logs and various device data points.
- **Remote Logging**: `monodeamon` continuously sends device logs to a `monoserve` instance.
- **Device Identification**: `monodeamon` sends a device ID (serial number if available) to `monoserve` for per-device log file organization.
- **Cross-Compilation**: `monocli` handles the cross-compilation of `monodeamon` for specified Android target architectures.
- **Configurable Server Target**: `monocli install` allows specifying the `monoserve` IP and port for `monodeamon`.
- **Configurable Server Binding**: `monoserve` can bind to a specific IP address and port via command-line arguments.
- **Stealthy Operation**: `monodeamon` runs as a background process without a visible application icon.
- **Comprehensive Device Information Dump**: The `dump` command generates an HTML report (using Tera templates and Chart.js) and raw data files.

## Installation

### Prerequisites

Before using the Mono project, ensure that you have the following installed:

- **Rust**: Install the Rust toolchain from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).
- **Android NDK**: Ensure that the Android NDK is installed and the `ANDROID_NDK_HOME` environment variable is set to its root path. `monocli` requires this to cross-compile the `monodeamon`.
- **ADB**: Ensure that Android Debug Bridge (ADB) is installed and available in your system's PATH.

### Cloning the Repository

```bash
git clone https://github.com/incredimo/mono.git
cd mono
```

## Usage

### Building

Build all release binaries (`monocli`, `monodeamon`, `monoserve`):
```bash
# Note: monodeamon direct build like this is for host testing if ever needed.
# For device deployment, it's cross-compiled by monocli during its 'install' command.
cargo build --release
```

### Running `monocli`

After building, `monocli` can be run from the `target/release` directory or using `cargo run`:
```bash
./target/release/monocli <command> [options]
# OR
cargo run --release --bin monocli -- <command> [options]
```

### Running `monoserve`

`monoserve` can be run from the `target/release` directory or using `cargo run`:
```bash
./target/release/monoserve [options]
# OR
cargo run --release --bin monoserve -- [options]
```
Default: `./target/release/monoserve --bind-address 0.0.0.0 --port 12345`

Available options for `monoserve`:
*   `-b, --bind-address <ADDRESS>`: Address to bind the server to (default: `0.0.0.0`).
*   `-p, --port <PORT>`: Port to listen on (default: `12345`).

### Commands (`monocli`)

-   **`install --target <ARCH> --server-ip <IP> --server-port <PORT>`**:
    Cross-compiles, installs, and starts the `monodeamon` on the connected Android device.
    *   `--target <ARCH>`: Specifies the Android target architecture.
        Available targets: `aarch64-linux-android` (default), `armv7-linux-androideabi`, `x86_64-linux-android`, `i686-linux-android`.
    *   `--server-ip <IP>`: The IP address of the `monoserve` instance (default: `127.0.0.1`).
    *   `--server-port <PORT>`: The port of the `monoserve` instance (default: `12345`).
-   **`remove`**: Stops and removes the `monodeamon` from the connected Android device.
-   **`check`**: Verifies if the device is rooted and whether the `monodeamon` is currently running.
-   **`dump`**: Captures exhaustive logs and device information, saving the data to a local directory.

### Examples

- **Starting `monoserve` on a specific IP and port:**
    ```bash
    cargo run --release --bin monoserve -- --bind-address 192.168.1.50 --port 54321
    ```

- **Installing `monodeamon` for `aarch64-linux-android` and connecting to `192.168.1.50:54321`:**

```bash
    cargo run --release --bin monocli -- install --target aarch64-linux-android --server-ip 192.168.1.50 --server-port 54321
    ```

- **Removing `monodeamon`**:

    ```bash
    cargo run --release --bin monocli -- remove
    ```

- **Checking device status**:

    ```bash
    cargo run --release --bin monocli -- check
    ```

- **Dumping logs and device info**:

    ```bash
    cargo run --release --bin monocli -- dump
    ```

## Device Information Captured

The `monocli dump` command captures a wide range of device information by executing various ADB shell commands, including (but not limited to):
- System properties (`getprop`, including SDK version, manufacturer, product name)
- Battery status (`dumpsys battery`)
- Installed packages (`pm list packages -f`)
- CPU and Memory information (`cat /proc/cpuinfo`, `cat /proc/meminfo`)
- Network configuration (`ip addr show`, `ip route show`, `netstat`, `getprop net.hostname`)
- Storage details (`df -h`, `mount`, `dumpsys diskstats`)
- Security status (`getenforce`, `getprop ro.adb.secure`, boot properties)
- Running processes and services (`ps -A`, `dumpsys activity services`, `dumpsys activity processes`)

## HTML Report
The `monocli dump` command generates a comprehensive HTML dashboard (`dashboard.html`) in the dump directory. This report visualizes key device metrics using Chart.js and provides detailed information in collapsible sections. The report is generated using Tera templates located in `monocli/templates/`.

## Persistence
`monocli` currently does not set up persistence for `monodeamon` on the Android device. The `monodeamon` process will run until the device reboots or the process is manually stopped. For persistent execution on rooted devices, manual setup of a boot script (e.g., via Magisk, `init.rc`, or other root-enabled mechanisms) is required to start `/data/local/tmp/monodeamon <server_ip> <server_port>` on boot.

## Experimental Directory
The `experimental` directory at the root of the repository contains other scripts or components not part of the core mono functionality, such as an example for `parler-tts`. These are for exploration and not integrated into the main `monocli`, `monodeamon`, or `monoserve` tools.

## Contributing

Contributions are welcome! If you find issues or have ideas for improvements, please open an issue or submit a pull request.

### Setting Up for Development

1.  Fork the repository and clone it locally.
2.  Make your changes in a feature branch.
3.  Test your changes thoroughly (e.g., `cargo check`, `cargo build`, `cargo clippy`, `cargo fmt`).
4.  Open a pull request with a detailed description of your changes.

## Future Enhancements

### Security Hardening
The current communication between `monodeamon` and `monoserve` is unencrypted and unauthenticated. Future enhancements should include:
*   Implementing TLS encryption for data in transit.
*   Adding an authentication mechanism (e.g., pre-shared keys or tokens) to ensure only authorized daemons can connect to the server.

### Comprehensive Testing
To improve robustness and reliability:
*   Develop unit tests for key logic in `monocli`, `monodeamon`, and `monoserve`.
*   Create an integration testing strategy to verify the end-to-end data pipeline.

### New Capabilities
Based on user feedback and project goals, future versions could explore:
*   Expanding `monodeamon`'s data collection capabilities beyond `logcat` (e.g., periodic polling of specific system metrics, app-specific data).
*   More advanced data analysis and visualization in the `monocli` report.
*   Support for different output formats from `monoserve` (e.g., JSON, direct database integration).
*   A web interface for `monoserve` to view live logs or manage connected devices.