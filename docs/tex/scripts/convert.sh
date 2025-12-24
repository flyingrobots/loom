#!/bin/bash
# Simple wrapper kept for backward compatibility.
# Prefer using scripts/md2tex.py which adds hashing, config, and safety rails.
# Usage (old): convert.sh <source_dir> <output_dir> [python_interpreter]

set -euo pipefail

SRC_DIR="${1:-}"
OUT_DIR="${2:-}"
PYTHON="${3:-python3}"

if [ -z "$SRC_DIR" ] || [ -z "$OUT_DIR" ]; then
    echo "Usage: convert.sh <source_dir> <output_dir> [python_interpreter]" >&2
    exit 1
fi

# Guard: refuse to write into docs/tex/sources unless FORCE=1
if [[ "$OUT_DIR" == *"docs/tex/sources"* || "$OUT_DIR" == *"docs/tex/sources/"* ]]; then
    if [ "${FORCE:-0}" != "1" ]; then
        echo "Refusing to write into docs/tex/sources without FORCE=1." >&2
        exit 1
    fi
fi

mkdir -p "$OUT_DIR"

echo "[convert.sh] Converting MD from $SRC_DIR to $OUT_DIR (deprecated; use md2tex.py)"

for md in "$SRC_DIR"/*.md; do
    [ -e "$md" ] || continue
    base_name=$(basename "$md" .md)
    safe_name=$(echo "$base_name" | tr ' ' '-' | tr '[:upper:]' '[:lower:]')
    out="$OUT_DIR/$safe_name.tex"
    echo "  $base_name -> $out"
    pandoc --from=markdown --to=latex --syntax-highlighting=idiomatic "$md" -o "$out"
done

if [ -f "scripts/clean-unicode.py" ]; then
    echo "Cleaning unicode in $OUT_DIR with $PYTHON..."
    "$PYTHON" scripts/clean-unicode.py "$OUT_DIR" || echo "[warn] clean-unicode failed; continuing"
fi
