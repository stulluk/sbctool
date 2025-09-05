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

### Prerequisites
- Rust 1.70+ and Cargo
- For Windows cross-compilation: `mingw-w64`

### Build Commands

```sh
# Clone repository
git clone https://github.com/stulluk/sbctool.git
cd sbctool

# Build for current platform
cargo build --release

# Cross-compile for Windows (from Linux)
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64  # Ubuntu/Debian
cargo build --release --target x86_64-pc-windows-gnu
```

### GitHub Actions

The project includes GitHub Actions for automated cross-platform builds:
- **Linux**: `x86_64-unknown-linux-gnu`
- **Windows**: `x86_64-pc-windows-msvc`

Build artifacts are available in the Actions tab.

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

### Platform-Specific
- `rusb`: USB device communication (Windows/Linux)
- `libssh2-sys`: SSH protocol implementation
- `openssl-sys`: Cryptographic operations

## 🧪 Testing

### Test Cases
1. **SSH**: Connect to SBC via SSH alias
2. **ADB USB**: Connect to Android device via USB
3. **ADB TCP**: Connect to Android device via network

### Test Commands
```sh
# SSH test
sbctool ssh khadas

# ADB USB test (Windows)
sbctool adb

# ADB TCP test
sbctool adb -s 192.168.1.215
```

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

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [adb_client](https://github.com/cocool97/adb_client) - Pure Rust ADB implementation
- [ssh2](https://github.com/rust-lang/ssh2-rs) - SSH client library
- [clap](https://github.com/clap-rs/clap) - Command-line argument parser