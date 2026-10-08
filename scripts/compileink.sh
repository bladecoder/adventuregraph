#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
: "${INKLECATE:=rinklecate}"
"$INKLECATE" -o "$repo_dir/assets/story.ink.json" "$repo_dir/ink/story.ink"
