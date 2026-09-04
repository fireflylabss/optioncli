#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PKGBUILD="$ROOT/packaging/aur/PKGBUILD"
SRCINFO="$ROOT/packaging/aur/.SRCINFO"
TAG="${1:?usage: bump.sh <version|vVersion>}"
VER="${TAG#v}"
TARBALL_URL="https://github.com/fireflylabss/optioncli/archive/refs/tags/v${VER}.tar.gz"

echo "==> waiting for $TARBALL_URL"
for _ in $(seq 1 12); do
  if curl -fsI "$TARBALL_URL" >/dev/null 2>&1; then break; fi
  sleep 5
done

echo "==> hashing tarball"
SHA="$(curl -fsSL "$TARBALL_URL" | sha256sum | awk '{print $1}')"
sed -i "s/^pkgver=.*/pkgver=${VER}/" "$PKGBUILD"
sed -i "s/^pkgrel=.*/pkgrel=1/" "$PKGBUILD"
sed -i "s/^sha256sums=.*/sha256sums=('${SHA}')/" "$PKGBUILD"

cat > "$SRCINFO" <<EOF
pkgbase = opt
	pkgdesc = Option family CLI: dispatch, doctor, install and system utilities
	pkgver = ${VER}
	pkgrel = 1
	url = https://github.com/fireflylabss/optioncli
	arch = x86_64
	license = Apache-2.0
	makedepends = cargo
	depends = gcc-libs
	depends = glibc
	options = !lto
	source = optioncli-${VER}.tar.gz::https://github.com/fireflylabss/optioncli/archive/refs/tags/v${VER}.tar.gz
	sha256sums = ${SHA}

pkgname = opt
EOF
echo "==> opt $VER · sha256=$SHA"
