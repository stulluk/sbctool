# sbctool

A CLI tool to collect information from various Single Board Computers (SBCs).

## Purpose

`sbctool` is a cross-platform command-line utility designed to run on a host PC (Linux, Windows, or macOS) and gather information from target SBCs. It provides a unified interface for interacting with different SBCs through various communication backends.

## Usage

The tool is executed with a subcommand specifying the backend to use, followed by any necessary connection details.

### SSH Backend

Connect to an SBC using SSH:

```sh
sbctool ssh <user@host>
```

Example:

```sh
sbctool ssh khadas
# Sample output
Connecting to khadas via SSH...
Authenticated with public key (from SSH config: /home/USER/.ssh/id_rsa)
Linux khadas-edge 6.12.1-edge-rockchip64 #1 SMP PREEMPT Fri Nov 22 14:30:26 UTC 2024 aarch64 aarch64 aarch64 GNU/Linux


Exit status: 0
```

### ADB Backend

Connect to an SBC using ADB.

For a single device connected via USB:

```sh
sbctool adb
# Sample output
No specific device serial provided, checking all connected devices...

--- Connecting to device: ohm80566015800b1e ---
Command successful for ohm80566015800b1e:
Linux localhost 5.15.170-android14-11-g0552e0fe0b84-ab17825 #1 SMP PREEMPT Thu Aug 14 06:55:09 UTC 2025 armv8l Toybox



--- Connecting to device: 192.168.1.215:5555 ---
Command successful for 192.168.1.215:5555:
Linux localhost 5.15.170-android14-11-g0552e0fe0b84-ab17825 #1 SMP PREEMPT Thu Aug 14 06:55:09 UTC 2025 armv8l Toybox
```

For a device connected over TCP/IP:

```sh
sbctool adb -s <device_serial>
```

Example:

```sh
sbctool adb -s 192.168.1.215:5555
# Sample output
Connecting to device 192.168.1.215:5555 via ADB...
Command successful for 192.168.1.215:5555:
Linux localhost 5.15.170-android14-11-g0552e0fe0b84-ab17825 #1 SMP PREEMPT Thu Aug 14 06:55:09 UTC 2025 armv8l Toybox
```

## Building

To build the project, you need to have Rust and Cargo installed.

1.  Clone the repository:
    ```sh
    git clone <repository-url>
    ```
2.  Navigate to the project directory:
    ```sh
    cd sbctool
    ```
3.  Build the project:
    ```sh
    cargo build --release
    ```
4.  The executable will be located at `target/release/sbctool`.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

1.  Fork the repository.
2.  Create a new branch (`git checkout -b feature/your-feature`).
3.  Make your changes.
4.  Commit your changes (`git commit -am 'Add some feature'`).
5.  Push to the branch (`git push origin feature/your-feature`).
6.  Create a new Pull Request.
