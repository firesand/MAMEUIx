# Maintainer: edo hikmahtiar <edohikmahtiar@me.com>
pkgname=mameuix
pkgver=0.1.6
pkgrel=1
pkgdesc="Modern GUI frontend for MAME arcade emulator with CLRMamePro Lite Mode and enhanced ROM verification"
arch=('x86_64')
url="https://github.com/firesand/MAMEUIx"
license=('MIT' 'OFL-1.1')
depends=('mame>=0.200' 'glibc' 'gcc-libs' 'libx11' 'libxcb' 'libxrandr' 'libxinerama' 'libxcursor' 'libxi')
makedepends=('rust>=1.85' 'pkgconf' 'zstd' 'cmake' 'ninja')
optdepends=('mame-roms: Game ROMs for MAME')
provides=('mameuix')
conflicts=('mameuix-git')
source=("$pkgname-$pkgver.tar.gz")
sha256sums=('SKIP')
validpgpkeys=()

prepare() {
    cd "$srcdir/$pkgname-$pkgver"
    export CARGO_HOME="$srcdir/cargo"
    
    # Ensure we have the latest dependencies
    cargo fetch --locked
    
    # Create build directory for better organization
    mkdir -p build
}

build() {
    cd "$srcdir/$pkgname-$pkgver"
    export CARGO_HOME="$srcdir/cargo"
    
    # Set environment variables for optimal build
    export ZSTD_LIB_DIR=/usr/lib
    export ZSTD_STATIC=0
    export RUSTFLAGS="-C link-arg=-lzstd"
    export CARGO_INCREMENTAL=0
    export CARGO_PROFILE_RELEASE_OPT_LEVEL=3
    export CARGO_PROFILE_RELEASE_LTO=true
    export CARGO_PROFILE_RELEASE_STRIP=true
    
    # Build with release optimizations and locked dependencies
    cargo build --release --locked --frozen
    
    # Verify the binary was created
    if [ ! -f "target/release/mameuix" ]; then
        echo "Error: Binary not found after build"
        exit 1
    fi
    
}

check() {
    cd "$srcdir/$pkgname-$pkgver"
    export CARGO_HOME="$srcdir/cargo"
    
    cargo test --release --locked --frozen
}

package() {
    cd "$srcdir/$pkgname-$pkgver"
    
    # Install binary
    install -Dm755 target/release/mameuix "$pkgdir/usr/bin/mameuix"
    
    # Install desktop file
    install -Dm644 mameuix.desktop "$pkgdir/usr/share/applications/mameuix.desktop"
    
    # Install icons (check if they exist first)
    if [ -f "assets/icons/16x16/mameuix.png" ]; then
        install -Dm644 assets/icons/16x16/mameuix.png "$pkgdir/usr/share/icons/hicolor/16x16/apps/mameuix.png"
    fi
    if [ -f "assets/icons/32x32/mameuix.png" ]; then
        install -Dm644 assets/icons/32x32/mameuix.png "$pkgdir/usr/share/icons/hicolor/32x32/apps/mameuix.png"
    fi
    if [ -f "assets/icons/48x48/mameuix.png" ]; then
        install -Dm644 assets/icons/48x48/mameuix.png "$pkgdir/usr/share/icons/hicolor/48x48/apps/mameuix.png"
    fi
    if [ -f "assets/icons/64x64/mameuix.png" ]; then
        install -Dm644 assets/icons/64x64/mameuix.png "$pkgdir/usr/share/icons/hicolor/64x64/apps/mameuix.png"
    fi
    if [ -f "assets/icons/128x128/mameuix.png" ]; then
        install -Dm644 assets/icons/128x128/mameuix.png "$pkgdir/usr/share/icons/hicolor/128x128/apps/mameuix.png"
    fi
    if [ -f "assets/icons/256x256/mameuix.png" ]; then
        install -Dm644 assets/icons/256x256/mameuix.png "$pkgdir/usr/share/icons/hicolor/256x256/apps/mameuix.png"
    fi
    if [ -f "assets/icons/scalable/mameuix.svg" ]; then
        install -Dm644 assets/icons/scalable/mameuix.svg "$pkgdir/usr/share/icons/hicolor/scalable/apps/mameuix.svg"
    fi
    
    # Install man page
    install -Dm644 debian/mameuix.1 "$pkgdir/usr/share/man/man1/mameuix.1"
    
    # Install documentation
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 CHANGELOG.md "$pkgdir/usr/share/doc/$pkgname/CHANGELOG.md"
    install -Dm644 LICENSE "$pkgdir/usr/share/doc/$pkgname/LICENSE"
    install -Dm644 assets/fonts/public_sans/OFL.txt "$pkgdir/usr/share/licenses/$pkgname/PublicSans-OFL.txt"
    
    # Install public guides (internal project notes are intentionally excluded)
    install -Dm644 docs/INSTALL.md "$pkgdir/usr/share/doc/$pkgname/INSTALL.md"
    install -Dm644 docs/MAME_FOLDER_STRUCTURE.md "$pkgdir/usr/share/doc/$pkgname/MAME_FOLDER_STRUCTURE.md"
    install -Dm644 docs/BGFX_GLSL_INTEGRATION.md "$pkgdir/usr/share/doc/$pkgname/BGFX_GLSL_INTEGRATION.md"
    
    # Install examples (if they exist)
    if [ -d "examples" ]; then
        install -dm755 "$pkgdir/usr/share/doc/$pkgname/examples"
        cp -r examples/* "$pkgdir/usr/share/doc/$pkgname/examples/" 2>/dev/null || true
    fi
    
    # Install shaders (if they exist)
    if [ -d "shaders" ]; then
        install -dm755 "$pkgdir/usr/share/doc/$pkgname/shaders"
        cp -r shaders/* "$pkgdir/usr/share/doc/$pkgname/shaders/" 2>/dev/null || true
    fi
    
    # Create configuration directory
    install -dm755 "$pkgdir/etc/mameuix"
    
    # Install default configuration files (if they exist)
    if [ -d "cfg" ]; then
        install -dm755 "$pkgdir/etc/mameuix/cfg"
        cp -r cfg/* "$pkgdir/etc/mameuix/cfg/" 2>/dev/null || true
    fi
} 
