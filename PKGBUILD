# Maintainer: olafkfreund <olafkfreund@gmail.com>
pkgname=r-hyprconfig
pkgver=1.2.0
pkgrel=1
pkgdesc="A modern TUI for visually configuring Hyprland"
arch=('x86_64' 'aarch64')
url="https://github.com/olafkfreund/r-hyprconfig"
license=('MIT')
depends=('gcc-libs' 'glibc')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/olafkfreund/r-hyprconfig/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')  # Will be updated when creating the actual AUR package

prepare() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo build --frozen --release --all-features
}

check() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo test --frozen --all-features
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm0755 -t "$pkgdir/usr/bin/" "target/release/$pkgname"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}