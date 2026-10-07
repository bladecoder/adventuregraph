#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
: "${INKLECATE:=inklecate}"
"$INKLECATE" -o "$repo_dir/assets/story.ink.json" "$repo_dir/raw/ink-src/TheIntercept.ink"
