#!/usr/bin/env bash
# onparmak kurulum betiği: son sürümü GitHub'dan indirip ~/.local/bin'e koyar.
#   curl -fsSL https://raw.githubusercontent.com/KULLANICI_ADI/onparmak/main/install.sh | bash
set -euo pipefail

REPO="${ONPARMAK_REPO:-KULLANICI_ADI/onparmak}"
BIN_DIR="${ONPARMAK_BIN_DIR:-$HOME/.local/bin}"

os="$(uname -s)"
arch="$(uname -m)"
case "$os/$arch" in
  Linux/x86_64) target="x86_64-unknown-linux-musl" ;;
  Linux/aarch64 | Linux/arm64) target="aarch64-unknown-linux-musl" ;;
  Darwin/arm64) target="aarch64-apple-darwin" ;;
  Darwin/x86_64) target="x86_64-apple-darwin" ;;
  *) echo "Desteklenmeyen sistem: $os/$arch" >&2; exit 1 ;;
esac

url="https://github.com/$REPO/releases/latest/download/onparmak-$target.tar.gz"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "İndiriliyor: $url"
curl -fsSL "$url" -o "$tmp/onparmak.tar.gz"
tar xzf "$tmp/onparmak.tar.gz" -C "$tmp"

mkdir -p "$BIN_DIR"
install -m 755 "$tmp/onparmak-$target/onparmak" "$BIN_DIR/onparmak"
echo "Kuruldu: $BIN_DIR/onparmak"

# Linux'ta uygulama başlatıcısına ekle (Omarchy'de Super+Space ile açılır).
if [[ "$os" == "Linux" ]]; then
  apps="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
  mkdir -p "$apps"
  sed "s|^Exec=onparmak|Exec=$BIN_DIR/onparmak|" "$tmp/onparmak-$target/onparmak.desktop" > "$apps/onparmak.desktop"
  echo "Başlatıcıya eklendi: $apps/onparmak.desktop"
fi

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) echo "Not: $BIN_DIR PATH'te değil. Kabuk ayarına ekle: export PATH=\"$BIN_DIR:\$PATH\"" ;;
esac
echo "Başlamak için: onparmak"
