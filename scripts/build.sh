#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mode="${1:-release}"
board="${ADVENTUREGRAPH_BOARD:-v1.1}"
if [[ $# -gt 1 ]]; then
    echo "usage: ADVENTUREGRAPH_BOARD=v1.1 scripts/build.sh [release|debug]" >&2
    exit 2
fi
export ADVENTUREGRAPH_BOARD="$board"
echo "Building board profile: $board"
if [[ "$mode" != release && "$mode" != debug ]]; then
    echo "usage: scripts/build.sh [release|debug]" >&2
    exit 2
fi
if ! command -v xtensa-esp32s2-elf-gcc >/dev/null; then
    esp_export="${ESPUP_EXPORT_FILE:-$HOME/export-esp.sh}"
    if [[ -f "$esp_export" ]]; then
        source "$esp_export"
    fi
fi
if ! command -v xtensa-esp32s2-elf-gcc >/dev/null; then
    echo "Xtensa linker missing; install espup and source its export file" >&2
    exit 1
fi
cd "$repo_dir/esp"
export RUSTUP_TOOLCHAIN=esp
if [[ "$mode" == release ]]; then
    cargo build --locked --release -p adventuregraph-esp
else
    cargo build --locked -p adventuregraph-esp
fi
