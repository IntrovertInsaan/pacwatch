#!/usr/bin/env bash
# Checks categories.toml against your ACTUAL installed packages.
# Run this on your Arch machine, not in any sandbox.
set -euo pipefail

TOML="${HOME}/.config/pacwatch/categories.toml"
if [[ ! -f "$TOML" ]]; then
  echo "No categories.toml found at $TOML" >&2
  exit 1
fi

comm -23 \
  <(pacman -Qq | sort -u) \
  <(grep -oE '"[a-zA-Z0-9._+-]+"' "$TOML" | tr -d '"' | sort -u) \
  >/tmp/pacwatch_missing.txt

count=$(wc -l </tmp/pacwatch_missing.txt)
echo "Packages installed but NOT in categories.toml: $count"
if [[ "$count" -gt 0 ]]; then
  echo "(these will show as 'Uncategorized' in pacwatch)"
  cat /tmp/pacwatch_missing.txt
fi
