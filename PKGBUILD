# Maintainer: Arne Fuchs
pkgname="edcas-client"
pkgver="0.4.0"
pkgrel="1"
pkgdesc="Elite Dangerous Commander Assistant System"
url="https://github.com/arne-fuchs/edcas-client"
arch=("x86_64")
license=("Apache-2.0")
depends=("gcc-libs" "glibc" "openssl")

source=(
    "${pkgname}-${pkgver}.tar.gz::${url}/releases/download/v${pkgver}/edcas-client-linux-x86_64.tar.gz"
)
sha256sums=("SKIP")

package() {
    cd "${srcdir}"
    install -Dm755 "edcas-client"          "${pkgdir}/usr/bin/${pkgname}"
    install -Dm644 "settings-example.json" "${pkgdir}/etc/${pkgname}/settings-example.json"
}
