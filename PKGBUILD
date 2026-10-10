# Maintainer: Viboscop
pkgname=viboscop
pkgver=0.1.0
pkgrel=1
pkgdesc="Native desktop stereo XY oscilloscope for WAV and FLAC audio files"
arch=("x86_64")
license=("MIT")
depends=("alsa-lib" "gcc-libs")
makedepends=("cargo" "rust" "git" "pkgconf")
source=("git+file://$PWD")
sha256sums=("SKIP")

prepare() {
  cd "$srcdir/$pkgname"
  cargo fetch --locked
}

build() {
  cd "$srcdir/$pkgname"
  cargo build --release --locked
}

package() {
  cd "$srcdir/$pkgname"
  install -Dm755 target/release/viboscop "$pkgdir/usr/bin/viboscop"
}
