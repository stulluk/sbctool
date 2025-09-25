# sbctool

A cross-platform CLI tool to collect information from various Single Board Computers (SBCs) via SSH and ADB.

## 🎯 Purpose

`sbctool` is a pure Rust command-line utility designed to run on host PCs (Linux, Windows, macOS) and gather information from target SBCs. It provides a unified interface for interacting with different SBCs through SSH and ADB backends, with **zero external binary dependencies**.

## ✨ Key Features

- **Cross-platform**: Works on Linux, Windows, and macOS
- **Pure Rust**: No external binary dependencies (no need for `ssh` or `adb` commands)
- **Multiple backends**: SSH and ADB support
- **Smart connection detection**: Automatic USB/TCP/Server mode detection
- **SSH alias support**: Resolves SSH config aliases using `ssh -G`
- **Direct USB ADB**: Native USB device communication on Windows and Linux
- **🆕 TUI Interface**: Real-time Text-based User Interface with system monitoring
- **🆕 Comprehensive System Info**: Chip detection, memory, uptime, OS information
- **🆕 Real-time Logs**: logcat (Android) and journald/syslog (Linux) streaming

## 🏗️ Architecture

### SSH Backend
- Uses `ssh2` crate for native SSH client implementation
- Supports SSH config file parsing (`~/.ssh/config`, `/etc/ssh/ssh_config`)
- Falls back to `ssh -G` for alias resolution when available
- Supports public key and password authentication

### ADB Backend
- Uses `adb_client` crate for pure Rust ADB implementation
- **Direct USB mode**: Native USB device communication (no ADB server required)
- **Direct TCP mode**: Direct connection to ADB daemon over network
- **ADB Server mode**: Uses existing ADB server when available
- Automatic mode detection based on connection parameters

## 🚀 Usage

### TUI Interface (New!)

`sbctool` now launches a real-time Text-based User Interface (TUI) for system monitoring:

```sh
# SSH connection with TUI
sbctool ssh <user@host|alias>

# ADB connection with TUI
sbctool adb [-s <serial>]
```

**TUI Features:**
- **Left Panel**: System information (chipset, CPU, memory, uptime, OS)
- **Right Panel**: Real-time logs (logcat for Android, journald/syslog for Linux)
- **Helper Bar**: Keyboard shortcuts at the bottom
- **Controls**: `q` or `ESC` to exit, `r` to refresh

### SSH Backend

Connect to an SBC using SSH:

```sh
sbctool ssh <user@host|alias>
```

**Examples:**
```sh
# Using IP address
sbctool ssh user@192.168.1.4

# Using SSH alias (resolved from ~/.ssh/config)
sbctool ssh khadas

# Help
sbctool ssh help
```

**Sample Output:**
```
Connecting to khadas via SSH...
Authenticated with public key (from SSH config: /home/USER/.ssh/id_rsa)
Linux khadas-edge 6.12.1-edge-rockchip64 #1 SMP PREEMPT Fri Nov 22 14:30:26 UTC 2024 aarch64 aarch64 aarch64 GNU/Linux

Exit status: 0
```

### ADB Backend

Connect to Android devices using ADB:

```sh
sbctool adb [-s SERIAL]
```

**Connection Modes:**
- **No `-s`**: Automatic detection (USB direct → ADB server fallback)
- **`-s <ip>`**: Direct TCP connection (default port 5555)
- **`-s <ip:port>`**: Direct TCP connection to specific port
- **`-s <usb-serial>`**: ADB server connection to specific device

**Examples:**
```sh
# Automatic detection (USB direct on Windows, ADB server on Linux)
sbctool adb

# Direct TCP connection
sbctool adb -s 192.168.1.215
sbctool adb -s 192.168.1.215:5555

# ADB server connection
sbctool adb -s ohm80566015800b1e

# Help
sbctool adb help
```

**Sample Outputs:**

*Windows (Direct USB):*
```
ADB USB (direct): found device 18d1:4ee7
Linux localhost 5.15.170-android14-11-g0552e0fe0b84-ab17825 #1 SMP PREEMPT Thu Aug 14 06:55:09 UTC 2025 armv8l Toybox
```

*Linux (Direct TCP):*
```
ADB TCP (direct): shell uname on 192.168.1.215:5555
Linux localhost 5.15.170-android14-11-g0552e0fe0b84-ab17825 #1 SMP PREEMPT Thu Aug 14 06:55:09 UTC 2025 armv8l Toybox
```

## 🔧 Building

### Quick Start

```sh
# Clone and build
git clone https://github.com/stulluk/sbctool.git
cd sbctool
cargo build --release
```

### Prerequisites

- **Rust**: 1.70+ with Cargo
- **System dependencies**: See [BUILDING.md](BUILDING.md) for detailed platform-specific requirements

### Build Commands

#### Current Platform
```sh
# Debug build
cargo build

# Release build
cargo build --release
```

#### Cross-Compilation
```sh
# Linux to Windows
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64  # Ubuntu/Debian
cargo build --release --target x86_64-pc-windows-gnu
```

#### Docker Build
```sh
# Build with Docker
docker build -t sbctool .

# Run with USB access
docker run --rm --privileged -v /dev/bus/usb:/dev/bus/usb sbctool adb
```

### GitHub Actions

Automated cross-platform builds:
- **Linux**: `x86_64-unknown-linux-gnu`
- **Windows**: `x86_64-pc-windows-msvc`

**Download binaries**: Go to [Actions tab](https://github.com/stulluk/sbctool/actions) → Latest build → Download artifacts

### Detailed Build Instructions

For comprehensive build instructions, troubleshooting, and advanced configuration, see [BUILDING.md](BUILDING.md).

## 📋 Platform Support

| Platform | SSH | ADB USB | ADB TCP | ADB Server |
|----------|-----|---------|---------|------------|
| Linux    | ✅  | ✅      | ✅      | ✅         |
| Windows  | ✅  | ✅      | ✅      | ✅*        |
| macOS    | ✅  | ✅      | ✅      | ✅         |

*Windows ADB Server requires Android SDK Platform Tools installation

## 🔌 Dependencies

### Core Dependencies
- `clap`: Command-line argument parsing
- `anyhow`: Error handling
- `ssh2`: SSH client implementation
- `ssh_config`: SSH configuration parsing
- `adb_client`: Pure Rust ADB client implementation

### TUI Dependencies
- `ratatui`: Text-based User Interface framework
- `crossterm`: Terminal manipulation and control
- `tokio`: Asynchronous runtime for real-time updates
- `serde`: Serialization framework
- `serde_json`: JSON serialization support
- `chrono`: Date and time handling

### Platform-Specific
- `rusb`: USB device communication (Windows/Linux)
- `libssh2-sys`: SSH protocol implementation
- `openssl-sys`: Cryptographic operations

## 📊 System Information Collection

### Linux SBC Support
- **Chip Detection**: Device tree parsing (`/proc/device-tree/model`, `/proc/device-tree/compatible`)
- **CPU Info**: ARM implementer codes and architecture detection
- **Memory**: Total system memory from `/proc/meminfo`
- **Uptime**: System uptime from `uptime` command
- **OS Info**: Distribution information from `/etc/os-release`

### Android Device Support
- **Chip Detection**: Device properties (`getprop ro.product.manufacturer`, `ro.product.model`)
- **CPU Info**: ARM architecture and core count
- **Memory**: Total memory from `free` command
- **Uptime**: System uptime from `uptime` command
- **OS Info**: Android version from `getprop ro.build.version.release`

### Supported Chipsets
- **Rockchip**: RK3399, RK3568, RK3588
- **Amlogic**: G12, S905, S922
- **Allwinner**: Various SoCs
- **Broadcom**: BCM series
- **Qualcomm**: Snapdragon series
- **Nvidia**: Jetson series

## 🧪 Testing

### Test Cases
1. **SSH**: Connect to SBC via SSH alias
2. **ADB USB**: Connect to Android device via USB
3. **ADB TCP**: Connect to Android device via network

### Test Commands
```sh
# SSH test (Khadas Edge-V - Rockchip RK3399)
sbctool ssh khadas

# ADB USB test (Windows)
sbctool adb

# ADB TCP test (OHM - Amlogic)
sbctool adb -s 192.168.1.215
```

### Tested Devices
- **Khadas Edge-V**: Rockchip RK3399, Ubuntu 22.04, 4GB RAM
- **OHM**: Amlogic, Android 14, 1.9GB RAM

## 🤝 Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

1. Fork the repository
2. Create a new branch (`git checkout -b feature/your-feature`)
3. Make your changes
4. Test on multiple platforms
5. Commit your changes (`git commit -am 'Add some feature'`)
6. Push to the branch (`git push origin feature/your-feature`)
7. Create a new Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [adb_client](https://github.com/cocool97/adb_client) - Pure Rust ADB implementation
- [ssh2](https://github.com/rust-lang/ssh2-rs) - SSH client library
- [clap](https://github.com/clap-rs/clap) - Command-line argument parser