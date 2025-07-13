# Maintainer: edo hikmahtiar <edohikmahtiar@me.com>
pkgname=mameuix
pkgver=0.1.1
pkgrel=1
pkgdesc="Modern GUI frontend for MAME arcade emulator"
arch=('x86_64')
url="https://github.com/firesand/MAMEUIx"
license=('MIT')
depends=('mame>=0.200' 'gtk3' 'webkit2gtk')
makedepends=('rust' 'cargo' 'pkgconf' 'openssl' 'gtk3' 'webkit2gtk')
source=("$pkgname-$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release
}

package() {
    cd "$pkgname-$pkgver"
    
    # Install binary
    install -Dm755 target/release/mameuix "$pkgdir/usr/bin/mameuix"
    
    # Install desktop file
    install -Dm644 mameuix.desktop "$pkgdir/usr/share/applications/mameuix.desktop"
    
    # Install icons
    install -Dm644 assets/icons/16x16/mameuix-icon.png "$pkgdir/usr/share/icons/hicolor/16x16/apps/mameuix.png"
    install -Dm644 assets/icons/32x32/mameuix-icon.png "$pkgdir/usr/share/icons/hicolor/32x32/apps/mameuix.png"
    install -Dm644 assets/icons/48x48/mameuix-icon.png "$pkgdir/usr/share/icons/hicolor/48x48/apps/mameuix.png"
    install -Dm644 assets/icons/64x64/mameuix-icon.png "$pkgdir/usr/share/icons/hicolor/64x64/apps/mameuix.png"
    install -Dm644 assets/icons/128x128/mameuix-icon.png "$pkgdir/usr/share/icons/hicolor/128x128/apps/mameuix.png"
    install -Dm644 assets/icons/256x256/mameuix-icon.png "$pkgdir/usr/share/icons/hicolor/256x256/apps/mameuix.png"
    install -Dm644 assets/icons/scalable/mameuix-icon.svg "$pkgdir/usr/share/icons/hicolor/scalable/apps/mameuix.svg"
    
    # Install man page
    install -Dm644 mameuix.1 "$pkgdir/usr/share/man/man1/mameuix.1"
    
    # Install documentation
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
} 
