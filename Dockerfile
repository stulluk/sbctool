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
