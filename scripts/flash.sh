#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mode="${1:-release}"
"$repo_dir/scripts/build.sh" "$mode"
if ! command -v espflash >/dev/null; then
    echo "espflash is required to flash the board" >&2
    exit 1
fi
espflash flash --chip esp32s2 --monitor "$repo_dir/target/xtensa-esp32s2-none-elf/$mode/adventuregraph-esp"
