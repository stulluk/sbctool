# Multi-stage build for sbctool (Linux + Windows binaries)
FROM rust:1.80-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libusb-1.0-0-dev \
    mingw-w64 \
    curl \
    cmake \
    nasm \
    && rm -rf /var/lib/apt/lists/*

# Install latest Rust nightly
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Build for Linux
RUN cargo build --release

# Build for Windows
RUN rustup target add x86_64-pc-windows-gnu
RUN cargo build --release --target x86_64-pc-windows-gnu

# Output stage - just copy binaries
FROM scratch as output
COPY --from=builder /app/target/release/sbctool /sbctool-linux
COPY --from=builder /app/target/x86_64-pc-windows-gnu/release/sbctool.exe /sbctool-windows.exe
