#!/usr/bin/env bash
set -euo pipefail

# Helpers

error() {
    printf "\033[0;31merro:\033[0m %s\n" "$*" >&2
    exit 1
}
info() { printf "\033[0;2m%s\033[0m\n" "$*"; }
success() { printf "\033[0;32m%s\033[0m\n" "$*"; }
need() { command -v "$1" >/dev/null || error "$1 é necessário"; }

detect_shell() {
    local sh=${SHELL##*/}
    [[ $sh =~ (bash|zsh|fish) ]] && {
        printf %s "$sh"
        return
    }
    [[ -r /proc/$$/cmdline ]] && head -c 128 /proc/$$/cmdline |
        tr '\0' ' ' | awk '{print $1}' | xargs basename
}

add_line() { grep -Fqx "$2" "$1" 2>/dev/null || printf '%s\n' "$2" >>"$1"; }

# Delegate to PowerShell on native Windows

if [[ ${OS:-} == Windows_NT && ! $(uname -s) =~ MINGW64 ]]; then
    powershell -NoLogo -Command \
        "iwr https://raw.githubusercontent.com/gabrielbrunop/tenda/main/scripts/install.ps1 -UseBasicParsing | iex"
    exit $?
fi

# Prerequisites

need curl
need tar
need uname

# Choose release

REPO="gabrielbrunop/tenda"
TAG="${1:-}"
if [[ -z $TAG ]]; then
    TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" |
        grep -oP '"tag_name":\s*"\K[^"]+') ||
        error "Não foi possível obter a última versão da Tenda"
fi
info "Instalando Tenda $TAG"

# Detect platform

kernel=$(uname -s)
arch=$(uname -m)
case "$kernel,$arch" in
Darwin,x86_64) asset="tenda-x86_64-apple-darwin.tar.gz" ;;
Darwin,arm64) asset="tenda-aarch64-apple-darwin.tar.gz" ;;
Linux,x86_64) asset="tenda-x86_64-unknown-linux-gnu.tar.gz" ;;
MINGW* | MSYS* | CYGWIN*)
    need unzip
    asset="tenda-x86_64-pc-windows-msvc.zip"
    ;;
*) error "Plataforma não suportada: $kernel $arch" ;;
esac
info "Plataforma detectada → $asset"

# Download asset

INSTALL_DIR="${TENDA_INSTALL:-$HOME/.tenda}"
BIN_DIR="$INSTALL_DIR/bin"
mkdir -p "$BIN_DIR"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

url="https://github.com/$REPO/releases/download/$TAG/$asset"
info "Baixando $url"
curl -#fL "$url" -o "$tmp/$asset" || error "Falha ao baixar"

# Unpack & install

case $asset in
*.tar.gz) tar -xzf "$tmp/$asset" -C "$tmp" ;;
*.zip) unzip -q "$tmp/$asset" -d "$tmp" ;;
esac
rm "$tmp/$asset"

BIN_NAME=$([[ $asset == *.zip ]] && echo "tenda.exe" || echo "tenda")
mv -f "$tmp/$BIN_NAME" "$BIN_DIR/$BIN_NAME" ||
    error "Falha ao mover o binário para $BIN_DIR"
chmod +x "$BIN_DIR/$BIN_NAME"
success "Tenda instalada em $BIN_DIR/$BIN_NAME"

# Update shell profile(s)

PROFILE_MODIFIED=""
add_to_profile() {
    local export_root="export TENDA_INSTALL=\"$INSTALL_DIR\""
    local export_path="export PATH=\"$BIN_DIR:\$PATH\""
    case $(detect_shell) in
    bash)
        for rc in "$HOME/.bashrc" "$HOME/.bash_profile" \
            "${XDG_CONFIG_HOME:-$HOME/.config}/bashrc"; do
            [[ -w $rc || ! -e $rc ]] || continue
            touch "$rc"
            add_line "$rc" "$export_root"
            add_line "$rc" "$export_path"
            PROFILE_MODIFIED=$rc
            break
        done
        ;;
    zsh)
        local rc="$HOME/.zshrc"
        [[ -w $rc || ! -e $rc ]] || rc="$HOME/.zprofile"
        touch "$rc"
        add_line "$rc" "$export_root"
        add_line "$rc" "$export_path"
        PROFILE_MODIFIED=$rc
        ;;
    fish)
        need fish_add_path # fish >= 3.2
        fish_add_path "$BIN_DIR" >/dev/null 2>&1
        set -Ux TENDA_INSTALL "$INSTALL_DIR"
        PROFILE_MODIFIED="fish_user_paths"
        ;;
    *)
        info "Shell desconhecido - adicione manualmente:"
        printf '  %s\n  %s\n' "$export_root" "$export_path"
        ;;
    esac
}
add_to_profile

# Finale

if command -v tenda >/dev/null 2>&1; then
    success "Execute 'tenda --ajuda' para começar!"
else
    echo
    info "Reincie seu terminal ou execute:"
    case $(detect_shell) in
    fish) info "  source ~/.config/fish/config.fish" ;;
    zsh) info "  exec \$SHELL" ;;
    *) info "  source ${PROFILE_MODIFIED:-<profile-file>}" ;;
    esac
fi
