pkgname=yoru
pkgver=0.1.1
pkgrel=1
pkgdesc="A modern AUR helper for Arch Linux"
arch=('x86_64' 'aarch64')
url="https://github.com/crucabena/yoru"
license=('MIT')
depends=('pacman' 'git' 'sudo')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
