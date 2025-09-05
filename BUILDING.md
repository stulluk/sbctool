# Building sbctool

This document provides comprehensive instructions for building `sbctool` from source on different platforms.

## Prerequisites

### System Requirements

- **Rust**: 1.70+ with Cargo
- **Git**: For cloning the repository
- **Platform-specific dependencies** (see below)

### Platform-Specific Dependencies

#### Linux (Ubuntu/Debian)
```bash
# Essential build tools
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# For cross-compilation to Windows
sudo apt install mingw-w64

# For USB support (if not already installed)
sudo apt install libusb-1.0-0-dev
```

#### Linux (CentOS/RHEL/Fedora)
```bash
# Essential build tools
sudo dnf groupinstall "Development Tools"
sudo dnf install pkg-config openssl-devel

# For cross-compilation to Windows
sudo dnf install mingw64-gcc

# For USB support
sudo dnf install libusb1-devel
```

#### macOS
```bash
# Install Xcode command line tools
xcode-select --install

# Install dependencies via Homebrew
brew install pkg-config openssl libusb
```

#### Windows
```bash
# Install Rust via rustup
# Download from: https://rustup.rs/

# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
```

## Building

### 1. Clone Repository
```bash
git clone https://github.com/stulluk/sbctool.git
cd sbctool
```

### 2. Build for Current Platform

#### Debug Build
```bash
cargo build
```

#### Release Build
```bash
cargo build --release
```

The executable will be located at:
- Debug: `target/debug/sbctool`
- Release: `target/release/sbctool`

### 3. Cross-Compilation

#### Linux to Windows
```bash
# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Install MinGW (Ubuntu/Debian)
sudo apt install mingw-w64

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu
```

The Windows executable will be at: `target/x86_64-pc-windows-gnu/release/sbctool.exe`

#### Linux to macOS
```bash
# Add macOS target
rustup target add x86_64-apple-darwin

# Install osxcross (requires macOS SDK)
# See: https://github.com/tpoechtrager/osxcross

# Build for macOS
cargo build --release --target x86_64-apple-darwin
```

## Docker Build

### Dockerfile
```dockerfile
# Multi-stage build for sbctool
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libusb-1.0-0-dev \
    mingw-w64 \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Build for Linux
RUN cargo build --release

# Build for Windows
RUN rustup target add x86_64-pc-windows-gnu
RUN cargo build --release --target x86_64-pc-windows-gnu

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libusb-1.0-0 \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=builder /app/target/release/sbctool /usr/local/bin/sbctool
COPY --from=builder /app/target/x86_64-pc-windows-gnu/release/sbctool.exe /usr/local/bin/sbctool.exe

# Set permissions
RUN chmod +x /usr/local/bin/sbctool

# Default command
CMD ["sbctool", "--help"]
```

### Building with Docker

#### Build Docker Image
```bash
# Build the image
docker build -t sbctool .

# Run the container
docker run --rm sbctool

# Run with USB device access (Linux)
docker run --rm --privileged -v /dev/bus/usb:/dev/bus/usb sbctool adb
```

#### Multi-Platform Docker Build
```bash
# Build for multiple platforms
docker buildx create --use
docker buildx build --platform linux/amd64,linux/arm64 -t sbctool:latest .
```

## GitHub Actions Build

The project includes automated builds via GitHub Actions:

### Workflow Features
- **Multi-platform builds**: Linux and Windows
- **Artifact upload**: Downloadable binaries
- **Cache optimization**: Faster subsequent builds
- **Matrix strategy**: Parallel builds

### Using GitHub Actions Builds

1. **Trigger Build**:
   - Push to `master` branch (automatic)
   - Manual trigger via GitHub Actions tab

2. **Download Artifacts**:
   - Go to Actions tab
   - Select latest successful build
   - Download `sbctool-linux` or `sbctool-windows` artifacts

3. **Local Download** (using GitHub CLI):
   ```bash
   # List recent runs
   gh run list --workflow="build.yml"
   
   # Download artifacts
   gh run download <run-id>
   ```

## Development Build

### Debug Build with Logging
```bash
# Enable debug logging
RUST_LOG=debug cargo run -- ssh khadas

# Enable trace logging for ADB
RUST_LOG=adb_client=trace cargo run -- adb
```

### Feature Flags
```bash
# Build with specific features
cargo build --features "vendored-openssl"

# Build without default features
cargo build --no-default-features
```

## Troubleshooting

### Common Issues

#### 1. SSL/TLS Errors
```bash
# Install OpenSSL development headers
sudo apt install libssl-dev  # Ubuntu/Debian
sudo dnf install openssl-devel  # CentOS/RHEL/Fedora
```

#### 2. USB Permission Errors (Linux)
```bash
# Add user to plugdev group
sudo usermod -a -G plugdev $USER

# Create udev rules for ADB devices
echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="18d1", MODE="0666", GROUP="plugdev"' | sudo tee /etc/udev/rules.d/51-android.rules

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

#### 3. Cross-Compilation Issues
```bash
# Set environment variables for MinGW
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++
export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar
```

#### 4. Windows Build Issues
```bash
# Install Windows SDK
# Download from: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/

# Set environment variables
set INCLUDE=C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\ucrt
set LIB=C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\ucrt\x64
```

### Build Optimization

#### Release Build Optimization
```bash
# Optimize for size
cargo build --release --config 'profile.release.opt-level = "z"'

# Optimize for speed
cargo build --release --config 'profile.release.opt-level = 3'
```

#### Link-Time Optimization
```bash
# Enable LTO
cargo build --release --config 'profile.release.lto = true'
```

## Testing

### Unit Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_ssh_connection

# Run with output
cargo test -- --nocapture
```

### Integration Tests
```bash
# Test SSH connection
cargo run -- ssh khadas

# Test ADB connection
cargo run -- adb -s 192.168.1.215

# Test with verbose output
RUST_LOG=debug cargo run -- adb
```

## Distribution

### Creating Release Packages

#### Linux
```bash
# Create tar.gz package
tar -czf sbctool-linux-x86_64.tar.gz target/release/sbctool

# Create deb package (requires cargo-deb)
cargo install cargo-deb
cargo deb
```

#### Windows
```bash
# Create zip package
zip sbctool-windows-x86_64.zip target/x86_64-pc-windows-gnu/release/sbctool.exe
```

### Static Linking
```bash
# Build statically linked binary (Linux)
cargo build --release --target x86_64-unknown-linux-musl
```

## Performance

### Build Time Optimization
```bash
# Use faster linker
cargo build --release --config 'target.x86_64-unknown-linux-gnu.linker = "clang"'

# Use parallel compilation
cargo build --release -j $(nproc)
```

### Runtime Performance
```bash
# Profile with perf (Linux)
perf record ./target/release/sbctool ssh khadas
perf report

# Profile with valgrind
valgrind --tool=callgrind ./target/release/sbctool ssh khadas
```

## Security

### Secure Builds
```bash
# Build with security flags
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release

# Audit dependencies
cargo install cargo-audit
cargo audit
```

### Code Signing (Windows)
```bash
# Sign Windows executable
signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com sbctool.exe
```
