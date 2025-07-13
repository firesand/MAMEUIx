# Maintainer: MAME Frontend Team <mame-frontend@example.com>
pkgname=mame-frontend
pkgver=0.1.1
pkgrel=1
pkgdesc="Modern GUI frontend for MAME arcade emulator"
arch=('x86_64')
url="https://github.com/yourusername/mame-frontend"
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
    install -Dm755 target/release/mame-frontend "$pkgdir/usr/bin/mame-frontend"
    
    # Install desktop file
    install -Dm644 mame-frontend.desktop "$pkgdir/usr/share/applications/mame-frontend.desktop"
    
    # Install icons
    install -Dm644 assets/icons/16x16/mame-frontend-icon.png "$pkgdir/usr/share/icons/hicolor/16x16/apps/mame-frontend.png"
    install -Dm644 assets/icons/32x32/mame-frontend-icon.png "$pkgdir/usr/share/icons/hicolor/32x32/apps/mame-frontend.png"
    install -Dm644 assets/icons/48x48/mame-frontend-icon.png "$pkgdir/usr/share/icons/hicolor/48x48/apps/mame-frontend.png"
    install -Dm644 assets/icons/64x64/mame-frontend-icon.png "$pkgdir/usr/share/icons/hicolor/64x64/apps/mame-frontend.png"
    install -Dm644 assets/icons/128x128/mame-frontend-icon.png "$pkgdir/usr/share/icons/hicolor/128x128/apps/mame-frontend.png"
    install -Dm644 assets/icons/256x256/mame-frontend-icon.png "$pkgdir/usr/share/icons/hicolor/256x256/apps/mame-frontend.png"
    install -Dm644 assets/icons/scalable/mame-frontend-icon.svg "$pkgdir/usr/share/icons/hicolor/scalable/apps/mame-frontend.svg"
    
    # Install man page
    install -Dm644 mame-frontend.1 "$pkgdir/usr/share/man/man1/mame-frontend.1"
    
    # Install documentation
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
} 