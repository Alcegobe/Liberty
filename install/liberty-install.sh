#!/bin/sh
# Liberty OS — bootstrap : transforme une Debian minimale en Liberty.
#
# Sur une Debian 12/13 fraîche (netinst, sans environnement de bureau),
# en root :
#
#   apt-get update && apt-get install -y curl ca-certificates
#   curl -fsSL https://raw.githubusercontent.com/Alcegobe/Liberty/main/install/liberty-install.sh | sh
#
# Variables optionnelles :
#   ANTHROPIC_API_KEY=sk-ant-...   clé API (sinon demandée interactivement)
#   LIBERTY_REF=main               branche/tag à installer
#   LIBERTY_USER=<login>           utilisateur dont le shell devient lish

set -eu

REPO="https://github.com/Alcegobe/Liberty.git"
REF="${LIBERTY_REF:-main}"
SRC=/opt/liberty/src

say() { printf '\033[1m[liberty]\033[0m %s\n' "$*"; }

[ "$(id -u)" = 0 ] || { echo "lance-moi en root." >&2; exit 1; }

say "1/6 — dépendances système"
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq --no-install-recommends \
    curl ca-certificates git build-essential pkg-config >/dev/null

say "2/6 — toolchain Rust"
if ! command -v cargo >/dev/null 2>&1 && [ ! -x "$HOME/.cargo/bin/cargo" ]; then
    curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal >/dev/null
fi
PATH="$HOME/.cargo/bin:$PATH"

say "3/6 — sources de Liberty ($REF)"
if [ -d "$SRC/.git" ]; then
    git -C "$SRC" fetch --depth 1 origin "$REF"
    git -C "$SRC" checkout -q FETCH_HEAD
else
    mkdir -p "$(dirname "$SRC")"
    git clone --depth 1 --branch "$REF" "$REPO" "$SRC"
fi

say "4/6 — compilation de l'esprit (libertyd + lish, features claude)"
cargo build --release --features claude \
    --manifest-path "$SRC/services/libertyd/Cargo.toml"
install -m 755 "$SRC/services/libertyd/target/release/libertyd" /usr/local/bin/
install -m 755 "$SRC/services/libertyd/target/release/lish" /usr/local/bin/

say "5/6 — configuration"
mkdir -p /etc/liberty /var/lib/liberty
if [ ! -f /etc/liberty/liberty.toml ]; then
    install -m 644 "$SRC/install/liberty.toml" /etc/liberty/liberty.toml
fi
if [ ! -s /etc/liberty/anthropic.key ]; then
    if [ -n "${ANTHROPIC_API_KEY:-}" ]; then
        printf '%s\n' "$ANTHROPIC_API_KEY" > /etc/liberty/anthropic.key
    else
        printf 'Clé API Anthropic (sk-ant-…, laisser vide pour plus tard) : '
        read -r KEY </dev/tty || KEY=""
        [ -n "$KEY" ] && printf '%s\n' "$KEY" > /etc/liberty/anthropic.key
    fi
fi
[ -f /etc/liberty/anthropic.key ] && chmod 600 /etc/liberty/anthropic.key

say "6/6 — service systemd + shell"
install -m 644 "$SRC/install/libertyd.service" /etc/systemd/system/libertyd.service
systemctl daemon-reload
systemctl enable libertyd.service >/dev/null 2>&1 || true
if [ -s /etc/liberty/anthropic.key ]; then
    systemctl restart libertyd.service
fi
grep -qx /usr/local/bin/lish /etc/shells || echo /usr/local/bin/lish >> /etc/shells
if [ -n "${LIBERTY_USER:-}" ] && id "$LIBERTY_USER" >/dev/null 2>&1; then
    chsh -s /usr/local/bin/lish "$LIBERTY_USER"
    say "shell de $LIBERTY_USER → lish"
fi

echo
say "Liberty est installé."
say "  esprit  : systemctl status libertyd   (journal : journalctl -u libertyd -f)"
say "  shell   : lish                        (langage naturel ; ! pour le shell brut)"
say "  config  : /etc/liberty/liberty.toml   (profil d'autonomie, capacités)"
if [ ! -s /etc/liberty/anthropic.key ]; then
    say "  ⚠ pas de clé API : mets-la dans /etc/liberty/anthropic.key puis"
    say "    systemctl restart libertyd"
fi
