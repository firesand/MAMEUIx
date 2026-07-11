#!/bin/bash
#
# Build a portable MAMEUIx AppImage (bundles mameuix + GUI libraries).
# MAME itself is NOT bundled — install separately or set the path in app settings.
#
# For best portability across distros, prefer building inside Ubuntu 22.04 (CI/container).
# Building on Gentoo works for local use but may target a newer glibc.

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Portable AppImages should be built on glibc <= 2.35 (Ubuntu 22.04).
# Building on newer hosts (e.g. Gentoo 2.43) breaks older distros at runtime.
MAX_PORTABLE_GLIBC="2.35"

glibc_version() {
    getconf GNU_LIBC_VERSION 2>/dev/null | awk '{print $2}'
}

glibc_ge() {
    # Returns 0 when $1 >= $2 (version compare).
    printf '%s\n%s\n' "$2" "$1" | sort -C -V
}

check_build_host_glibc() {
    if [[ "${SKIP_GLIBC_CHECK:-0}" == "1" ]]; then
        return 0
    fi

    local host_glibc
    host_glibc=$(glibc_version || true)
    if [[ -z "${host_glibc}" ]]; then
        return 0
    fi

    if glibc_ge "${host_glibc}" "${MAX_PORTABLE_GLIBC}" && [[ "${host_glibc}" != "${MAX_PORTABLE_GLIBC}" ]]; then
        print_warning "Host glibc ${host_glibc} is newer than portable target ${MAX_PORTABLE_GLIBC}."
        print_warning "AppImages built here may fail on older Linux (e.g. GLIBC_2.43 not found)."
        print_warning "Use one of:"
        print_warning "  ./build-appimage-docker.sh     # Ubuntu 22.04 container"
        print_warning "  gh workflow run appimage.yml -f build_tag=v${VERSION} -f upload_to_release=false"
        print_error "Refusing to build. Set SKIP_GLIBC_CHECK=1 to override (not recommended for release)."
        exit 1
    fi
}

if [[ ! -f Cargo.toml ]]; then
    print_error "Cargo.toml not found. Run this script from the project root."
    exit 1
fi

VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
ARCH=$(uname -m)
BUILD_TARGET_DIR="${CARGO_TARGET_DIR:-target}"
APPIMAGE_NAME="MAMEUIx-${VERSION}-${ARCH}.AppImage"
APPDIR="AppDir"
TOOLS_DIR="${TOOLS_DIR:-${HOME}/.cache/mameuix-appimage-tools}"
LINUXDEPLOY="${TOOLS_DIR}/linuxdeploy-${ARCH}.AppImage"

download_if_missing() {
    local url="$1"
    local dest="$2"

    if [[ -f "${dest}" ]]; then
        return 0
    fi

    print_status "Downloading $(basename "${dest}")..."
    mkdir -p "$(dirname "${dest}")"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "${dest}" "${url}"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "${dest}" "${url}"
    else
        print_error "Need curl or wget to download AppImage tooling."
        exit 1
    fi

    chmod +x "${dest}"
}

ensure_tools() {
    case "${ARCH}" in
        x86_64|aarch64) ;;
        *)
            print_error "Unsupported architecture for AppImage build: ${ARCH}"
            exit 1
            ;;
    esac

    download_if_missing \
        "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-${ARCH}.AppImage" \
        "${LINUXDEPLOY}"
}

prepare_icon() {
    local icon_dir="${APPDIR}/usr/share/icons/hicolor/256x256/apps"
    mkdir -p "${icon_dir}"

    if [[ -f assets/mameuix.png ]]; then
        cp assets/mameuix.png "${icon_dir}/mameuix.png"
    elif command -v rsvg-convert >/dev/null 2>&1 && [[ -f assets/mameuix.svg ]]; then
        rsvg-convert -w 256 -h 256 assets/mameuix.svg -o "${icon_dir}/mameuix.png"
    elif command -v convert >/dev/null 2>&1 && [[ -f assets/mameuix.svg ]]; then
        convert -background none assets/mameuix.svg -resize 256x256 "${icon_dir}/mameuix.png"
    else
        print_error "No icon found. Expected assets/mameuix.png or assets/mameuix.svg with rsvg-convert/convert."
        exit 1
    fi

    ln -sf usr/share/icons/hicolor/256x256/apps/mameuix.png "${APPDIR}/mameuix.png"
    ln -sf usr/share/icons/hicolor/256x256/apps/mameuix.png "${APPDIR}/.DirIcon"
}

build_binary() {
    if ! command -v cargo >/dev/null 2>&1; then
        print_error "cargo not found. Install Rust >= 1.85."
        exit 1
    fi

    print_status "Building mameuix ${VERSION} (release)..."
    cargo build --release --locked --bin mameuix
}

assemble_appdir() {
    print_status "Assembling AppDir..."
    rm -rf "${APPDIR}"

    mkdir -p "${APPDIR}/usr/bin"
    mkdir -p "${APPDIR}/usr/share/applications"
    mkdir -p "${APPDIR}/usr/share/doc/mameuix"

    cp "${BUILD_TARGET_DIR}/release/mameuix" "${APPDIR}/usr/bin/"
    chmod +x "${APPDIR}/usr/bin/mameuix"
    cp mameuix.desktop "${APPDIR}/usr/share/applications/"
    cp README.md CHANGELOG.md LICENSE "${APPDIR}/usr/share/doc/mameuix/" 2>/dev/null || true
    cp docs/INSTALL.md docs/MAME_FOLDER_STRUCTURE.md docs/BGFX_GLSL_INTEGRATION.md \
        "${APPDIR}/usr/share/doc/mameuix/"
    cp assets/fonts/public_sans/OFL.txt \
        "${APPDIR}/usr/share/doc/mameuix/PublicSans-OFL.txt"

    prepare_icon
}

create_appimage() {
    print_status "Bundling libraries with linuxdeploy..."
    export ARCH
    export LINUXDEPLOY_OUTPUT_VERSION="${VERSION}"
    export APPIMAGE_EXTRACT_AND_RUN=1

    # Remove stale output from previous runs.
    rm -f "${APPIMAGE_NAME}" MAMEUIx-*.AppImage

    "${LINUXDEPLOY}" --appdir "${APPDIR}" \
        --executable "${APPDIR}/usr/bin/mameuix" \
        --desktop-file "${APPDIR}/usr/share/applications/mameuix.desktop" \
        --icon-file "${APPDIR}/mameuix.png" \
        --output appimage

  local built
    built=$(ls -1 MAMEUIx-*.AppImage 2>/dev/null | head -1 || true)
    if [[ -z "${built}" ]]; then
        print_error "linuxdeploy did not produce an AppImage."
        exit 1
    fi

    if [[ "${built}" != "${APPIMAGE_NAME}" ]]; then
        mv -f "${built}" "${APPIMAGE_NAME}"
    fi

    chmod +x "${APPIMAGE_NAME}"
}

main() {
    print_status "MAMEUIx AppImage Builder"
    print_status "========================"

    check_build_host_glibc
    ensure_tools
    build_binary
    assemble_appdir
    create_appimage

    print_success "AppImage built: ${APPIMAGE_NAME}"
    ls -lh "${APPIMAGE_NAME}"
    print_warning "MAME is not included. Install MAME separately and set its path in MAMEUIx settings."
    print_status "Run: ./${APPIMAGE_NAME}"
    print_status "Redesign preview: ./${APPIMAGE_NAME} --redesign"
    print_status "If FUSE is unavailable: APPIMAGE_EXTRACT_AND_RUN=1 ./${APPIMAGE_NAME}"
}

main "$@"
