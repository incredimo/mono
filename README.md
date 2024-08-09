# mono

Mono is a Rust-based automated device data extraction and persistent remote logging mechanism designed for Android devices. This project provides a comprehensive toolset for managing a background daemon that captures logs and detailed device information remotely, offering a robust solution for continuous monitoring and data retrieval. PS: this was cooked up in less than an hour, so things might be a bit rough around the edges.😁

## Table of Contents

- [mono](#mono)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Features](#features)
  - [Installation](#installation)
    - [Prerequisites](#prerequisites)
    - [Cloning the Repository](#cloning-the-repository)
  - [Usage](#usage)
    - [Commands](#commands)
    - [Building and Running](#building-and-running)
      - [Building](#building)
      - [Running `monocli`](#running-monocli)
    - [Examples](#examples)
  - [Device Information Captured](#device-information-captured)
  - [Persistence and Stealth](#persistence-and-stealth)
    - [Persistence](#persistence)
    - [Stealth Mode](#stealth-mode)
  - [Contributing](#contributing)
    - [Setting Up for Development](#setting-up-for-development)

## Overview

Mono consists of three main components:

1. **monocli**: A command-line interface (CLI) for automating the installation, removal, checking, and data dumping operations related to the `monodeamon` on Android devices via ADB.
2. **monodeamon**: A Rust-based daemon that runs persistently on Android devices, continuously capturing logs and system data, which can be retrieved remotely.
3. **monoserve**: A Rust-based server that collects logs and data from multiple devices running `monodeamon`, providing centralized logging and monitoring.

## Features

- **Automated Data Extraction**: Mono automates the process of capturing logs and device data, organizing them for easy retrieval and analysis.
- **Persistent Logging**: The `monodeamon` ensures continuous log capture on the device, even when the device is not connected to a host machine (persistence is currently supported only on rooted devices).
- **Stealth Mode**: `monodeamon` runs as a background process without creating any visible Android application, making it undetectable through regular user interfaces.
- **Root Status Check**: The CLI can verify if the connected Android device is rooted, which can influence installation options.
- **Comprehensive Device Information Dump**: Mono can dump detailed system information, logs, and more into a structured directory, providing a snapshot of the device's state.
- **Automatic NDK Detection**: On Windows systems, Mono automatically detects the Android NDK installation path, simplifying the setup process.

## Installation

### Prerequisites

Before using the Mono project, ensure that you have the following installed:

- **Rust**: Install the Rust toolchain from [here](https://www.rust-lang.org/tools/install).
- **Android NDK**: Ensure that the Android NDK is installed. Mono automatically detects common NDK installation paths on Windows.
- **ADB**: Ensure that Android Debug Bridge (ADB) is installed and available in your system's PATH.

### Cloning the Repository

```bash
git clone https://github.com/incredimo/mono.git
cd mono
```

## Usage

Mono provides a CLI (`monocli`) that allows you to manage the `monodeamon` on connected Android devices. `monocli` supports several commands to facilitate device monitoring, log retrieval, and daemon management.

### Commands

- **install**: Installs and starts the `monodeamon` on the connected Android device, enabling persistent logging and data capture.
- **remove**: Safely removes the `monodeamon` from the connected Android device.
- **check**: Verifies if the device is rooted and whether the `monodeamon` is currently running.
- **dump**: Captures exhaustive logs and device information, saving the data to a local directory named after the device.

### Building and Running

#### Building

Build all components (`monocli`, `monodeamon`, and `monoserve`) using:

```bash
cargo build --release
```

#### Running `monocli`

After building the project, you can use `monocli` to interact with the connected Android device:

```bash
cargo run --release --bin monocli -- <command>
```

Replace `<command>` with one of the available commands (`install`, `remove`, `check`, `dump`).

### Examples

- **Installing `monodeamon`**:

    ```bash
    cargo run --release --bin monocli -- install
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

When using the `dump` command, Mono captures a wide range of device information, including but not limited to:

- **System Properties**: Captures all system properties using `adb shell getprop`.
- **Battery Status**: Gathers detailed battery information with `adb shell dumpsys battery`.
- **Installed Packages**: Lists all installed packages on the device using `adb shell pm list packages`.
- **CPU Information**: Dumps CPU information from `/proc/cpuinfo`.
- **Memory Information**: Dumps memory usage and statistics from `/proc/meminfo`.
- **Window Information**: Captures window manager information using `adb shell dumpsys window`.
- **Activity Manager State**: Captures the state of the activity manager using `adb shell dumpsys activity`.
- **Power Manager State**: Retrieves the power manager's state using `adb shell dumpsys power`.
- **Logcat Output**: Captures the current logs using `adb logcat -d`.

All this information is organized and saved into a directory named after the device model, providing a comprehensive snapshot of the device's current state.

## Persistence and Stealth

### Persistence

- **Rooted Devices**: Mono's `monodeamon` is designed to run persistently on rooted devices. Once installed, it will automatically restart upon device reboot, ensuring continuous monitoring and logging.
- **Non-Rooted Devices**: Persistence is limited on non-rooted devices due to Android's security restrictions. On non-rooted devices, `monodeamon` can be started manually using `adb`, but it will not survive a reboot.

### Stealth Mode

- **Invisible Operation**: The `monodeamon` runs as a background process and does not create any visible Android application or icon. This makes it undetectable through regular user interfaces, providing a stealthy logging solution.

## Contributing

Contributions are welcome! If you find issues or have ideas for improvements, please open an issue or submit a pull request.

### Setting Up for Development

1. Fork the repository and clone it locally.
2. Make your changes in a feature branch.
3. Test your changes thoroughly.
4. Open a pull request with a detailed description of your changes.

 