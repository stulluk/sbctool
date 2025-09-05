#!/bin/bash

# sbctool Docker Binary Build Script
# This script builds sbctool for both Linux and Windows using Docker

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
IMAGE_NAME="sbctool-binaries"
IMAGE_TAG="latest"
BUILD_ONLY=false
EXTRACT_BINARIES=true

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -n, --name NAME     Docker image name (default: sbctool-binaries)"
    echo "  -t, --tag TAG       Docker image tag (default: latest)"
    echo "  -b, --build-only    Only build, don't extract binaries"
    echo "  -h, --help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                           # Build and extract binaries"
    echo "  $0 -n my-sbctool -t v1.0    # Build with custom name and tag"
    echo "  $0 -b                        # Build only, no binary extraction"
    echo ""
    echo "This script builds sbctool for both Linux and Windows using Docker,"
    echo "then extracts the binaries to the current directory."
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--name)
            IMAGE_NAME="$2"
            shift 2
            ;;
        -t|--tag)
            IMAGE_TAG="$2"
            shift 2
            ;;
        -b|--build-only)
            BUILD_ONLY=true
            EXTRACT_BINARIES=false
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

FULL_IMAGE_NAME="${IMAGE_NAME}:${IMAGE_TAG}"

# Check if Dockerfile exists
if [ ! -f "Dockerfile" ]; then
    print_error "Dockerfile not found in current directory"
    exit 1
fi

print_status "Building Docker image: ${FULL_IMAGE_NAME}"
print_status "This will build sbctool for both Linux and Windows..."

# Build Docker image
if docker build --target output -t "${FULL_IMAGE_NAME}" .; then
    print_success "Docker image built successfully: ${FULL_IMAGE_NAME}"
else
    print_error "Failed to build Docker image"
    exit 1
fi

# Show image info
print_status "Image information:"
docker images "${FULL_IMAGE_NAME}"

if [ "$EXTRACT_BINARIES" = false ]; then
    print_status "Build-only mode: skipping binary extraction"
    exit 0
fi

print_status "Extracting binaries..."

# Create temporary container
CONTAINER_ID=$(docker create "${FULL_IMAGE_NAME}")
if [ $? -ne 0 ]; then
    print_error "Failed to create container for binary extraction"
    exit 1
fi

# Extract Linux binary
print_status "Extracting Linux binary..."
docker cp "${CONTAINER_ID}:/sbctool-linux" ./sbctool-docker-linux
if [ $? -ne 0 ]; then
    print_error "Failed to extract Linux binary"
    docker rm "${CONTAINER_ID}"
    exit 1
fi

# Extract Windows binary
print_status "Extracting Windows binary..."
docker cp "${CONTAINER_ID}:/sbctool-windows.exe" ./sbctool-docker-windows.exe
if [ $? -ne 0 ]; then
    print_error "Failed to extract Windows binary"
    docker rm "${CONTAINER_ID}"
    exit 1
fi

# Clean up container
docker rm "${CONTAINER_ID}"

# Set permissions
chmod +x ./sbctool-docker-linux

print_success "Binaries extracted successfully:"
echo "  - Linux: ./sbctool-docker-linux"
echo "  - Windows: ./sbctool-docker-windows.exe"

# Show file sizes
print_status "Binary sizes:"
ls -lh ./sbctool-docker-linux ./sbctool-docker-windows.exe

print_success "Docker build and extraction completed!"
print_status "You can now test the binaries:"
echo "  Linux: ./sbctool-docker-linux --help"
echo "  Windows: Copy sbctool-docker-windows.exe to Windows and run it"