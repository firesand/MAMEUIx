# MAMEUIx Installation

MAMEUIx v0.1.7 is supported on Linux. MAME is a separate runtime dependency and is not bundled with the application. FreeBSD amd64 is currently an experimental source-build target; see [Experimental FreeBSD](#experimental-freebsd).

## AppImage (recommended for Linux)

Download the current AppImage from [GitHub Releases](https://github.com/firesand/MAMEUIx/releases), then run:

```bash
chmod +x MAMEUIx-*.AppImage
./MAMEUIx-*.AppImage
```

The redesigned UI is an opt-in preview. Enable it from **Preferences → UI shell**, use the **Launch Redesign Preview** desktop action after integration, or start it directly:

```bash
./MAMEUIx-*.AppImage --redesign
```

Verify the downloaded artifact when the checksum file is available:

```bash
sha256sum -c MAMEUIx-*.AppImage.sha256
```

Install MAME with your distribution package manager first. If MAME is not on `PATH`, select it under **Options → Directories & Paths**. On Debian and Ubuntu it is commonly installed as `/usr/games/mame`.

If FUSE is unavailable:

```bash
APPIMAGE_EXTRACT_AND_RUN=1 ./MAMEUIx-*.AppImage
```

Official AppImages are built on Ubuntu 22.04 to remain compatible with glibc 2.35 and newer.

## Build from source on Linux

Requirements:

- Rust 1.85 or newer and Cargo
- MAME 0.200 or newer
- Git, CMake, `pkg-config`, and a C/C++ toolchain
- X11/XCB development libraries; Wayland libraries are recommended

### Debian and Ubuntu

```bash
sudo apt update
sudo apt install build-essential curl pkg-config cmake git mame \
    libx11-dev libx11-xcb-dev libxcb1-dev libxcb-render0-dev \
    libxcb-shape0-dev libxcb-xfixes0-dev libxrandr-dev \
    libxinerama-dev libxcursor-dev libxi-dev libxkbcommon-dev \
    libwayland-dev
```

### Fedora and related distributions

```bash
sudo dnf install gcc gcc-c++ curl pkgconf-pkg-config cmake git mame \
    libxcb-devel libxkbcommon-devel libX11-devel libXcursor-devel \
    libXi-devel libXinerama-devel libXrandr-devel wayland-devel
```

### Arch Linux and related distributions

```bash
sudo pacman -S --needed base-devel curl rust pkgconf cmake ninja mame \
    libx11 libxcb libxrandr libxinerama libxcursor libxi \
    libxkbcommon wayland
```

### Compile and run

```bash
git clone https://github.com/firesand/MAMEUIx.git
cd MAMEUIx
cargo build --release --locked
./target/release/mameuix
```

The `install*.sh` files are Linux-only convenience scripts. The AppImage, native-package, or source-build workflows documented here are preferred because their dependencies and outputs are explicit.

## Build native Linux packages

Run package builders from the repository root.

### Debian package

```bash
sudo apt install devscripts debhelper
./build-deb.sh
sudo apt install ../mameuix_*.deb
```

### RPM package

```bash
sudo dnf install rpm-build
./build-rpm.sh
sudo dnf install ./mameuix-*.rpm
```

### Arch package

The Arch builder creates an isolated source tree so it does not conflict with the repository's own `src/` directory.

```bash
./build-arch-package.sh
sudo pacman -U ./mameuix-*.pkg.tar.zst
```

Optional validation:

```bash
namcap PKGBUILD
namcap mameuix-*.pkg.tar.zst
```

### Gentoo

MAMEUIx is available from the [EDORP overlay](https://github.com/firesand/edorp-overlay):

```bash
emerge --sync edorp
emerge -av app-emulation/mameuix
```

## Experimental FreeBSD

FreeBSD amd64 is technically feasible because Rust provides the `x86_64-unknown-freebsd` target, MAME is available from FreeBSD Ports, and the GUI dependency stack contains FreeBSD/X11 paths. It is not an officially supported platform yet because MAMEUIx has not passed a native FreeBSD build and runtime smoke test.

Install the likely build and runtime dependencies:

```sh
pkg install git rust cmake pkgconf mame mesa-libs \
    libX11 libXcursor libXi libXinerama libXrandr libxcb \
    libxkbcommon wayland
```

Then build from source:

```sh
git clone https://github.com/firesand/MAMEUIx.git
cd MAMEUIx
cargo build --release --locked
cargo test --locked
./target/release/mameuix
```

MAMEUIx searches `/usr/local/bin/mame`, the standard FreeBSD package location. Use the `Auto` or `OpenGL` graphics backend initially. File dialogs may also require a working D-Bus/XDG Desktop Portal setup.

Before FreeBSD can be promoted from experimental to supported, it still needs native verification of the UI, file picker, `mame -listxml`, ROM scanning, verification, game launch, report opening, and BGFX/OpenGL paths. OpenBSD, NetBSD, and DragonFly BSD remain untested and must not be assumed compatible from the FreeBSD result.

References: [Rust FreeBSD target](https://doc.rust-lang.org/rustc/platform-support/freebsd.html), [FreeBSD MAME port](https://github.com/freebsd/freebsd-ports/tree/main/emulators/mame).

## Initial configuration

After installation:

1. Open **Options → Directories & Paths**.
2. Select the MAME executable.
3. Add ROM and CHD directories.
4. Configure artwork and optional support files such as `catver.ini` and `history.xml`.
5. Rescan the game list.

See [MAME folder structure](MAME_FOLDER_STRUCTURE.md) for the supported directory types.

## Troubleshooting

### MAME is not detected

Confirm that it runs independently:

```bash
mame -version
```

Then select the executable manually. Common locations include `/usr/games/mame`, `/usr/bin/mame`, and `/usr/local/bin/mame`.

### Build fails

```bash
rustc --version
cargo clean
cargo build --release --locked
```

Check the first native-library or `pkg-config` error and install the matching development package for your operating system.

### AppImage reports a missing GLIBC version

Use the AppImage attached to the GitHub release. Locally built AppImages inherit the builder's glibc requirements and may not run on older distributions.

For usage information, see the main [README](../README.md).
