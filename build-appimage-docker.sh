#!/bin/bash
#
# Build a portable MAMEUIx AppImage inside Ubuntu 22.04 (glibc 2.35).
# Use this instead of ./build-appimage.sh on bleeding-edge hosts (e.g. Gentoo)
# so the AppImage runs on older Linux distributions.
#
# Requires: docker or podman

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGE="${APPIMAGE_BUILD_IMAGE:-ubuntu:22.04}"

if [[ ! -f "${PROJECT_ROOT}/Cargo.toml" ]]; then
    print_error "Cargo.toml not found."
    exit 1
fi

RUNTIME=""
if command -v podman >/dev/null 2>&1; then
    RUNTIME=podman
elif command -v docker >/dev/null 2>&1; then
    RUNTIME=docker
else
    print_error "Need docker or podman. Alternatively, trigger the GitHub Actions workflow:"
    print_error "  gh workflow run appimage.yml"
    exit 1
fi

print_status "Building portable AppImage in ${IMAGE} via ${RUNTIME}..."

"${RUNTIME}" run --rm \
    -v "${PROJECT_ROOT}:/work:Z" \
    -w /work \
    -e APPIMAGE_EXTRACT_AND_RUN=1 \
    -e SKIP_GLIBC_CHECK=1 \
    "${IMAGE}" \
    bash -lc '
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive

apt-get update -qq
apt-get install -y -qq \
    curl ca-certificates file \
    build-essential pkg-config cmake \
    libx11-dev libx11-xcb-dev libxcb1-dev libxcb-render0-dev \
    libxcb-shape0-dev libxcb-xfixes0-dev libxrandr-dev \
    libxinerama-dev libxcursor-dev libxi-dev libxkbcommon-dev \
    libwayland-dev libssl-dev

if ! command -v cargo >/dev/null 2>&1; then
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
fi
source "${HOME}/.cargo/env"
rustc --version

./build-appimage.sh
'

print_success "Portable AppImage ready in ${PROJECT_ROOT}"
ls -lh "${PROJECT_ROOT}"/MAMEUIx-*.AppImage
