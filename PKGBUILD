# Maintainer: Mohan <mohan@archlinux.local>
pkgname=dropbar-git
pkgver=0.1.0
pkgrel=1
pkgdesc="Floating Drag & Drop bar and cross-device sync between Arch Linux (Hyprland) and Mobile"
arch=('x86_64')
url="https://github.com/dropbar/dropbar"
license=('MIT')
depends=('gtk4' 'libnotify')
makedepends=('cargo' 'git')
provides=('dropbar')
conflicts=('dropbar')
source=("dropbar::git+file://$PWD")
sha256sums=('SKIP')

build() {
    cd "$srcdir/dropbar"
    cargo build --release --locked
}

package() {
    cd "$srcdir/dropbar"
    install -Dm755 target/release/dropbar-desktop "$pkgdir/usr/bin/dropbar-desktop"
    install -Dm755 target/release/dropbar-cli "$pkgdir/usr/bin/dropbar-cli"
    install -Dm644 hyprland/dropbar.conf "$pkgdir/usr/share/dropbar/hyprland.conf"
    install -Dm644 systemd/dropbar.service "$pkgdir/usr/lib/systemd/user/dropbar.service"
}

