#!/bin/bash
# Usage: convert.sh <source_dir> <output_dir> [python_interpreter]
# Example: convert.sh ../ARCH build/chapters/arch venv/bin/python3

SRC_DIR="$1"
OUT_DIR="$2"
PYTHON="$3"

mkdir -p "$OUT_DIR"

echo "Converting MD from $SRC_DIR to $OUT_DIR..."

for md in "$SRC_DIR"/*.md; do
    [ -e "$md" ] || continue
    
    # Get base filename
    base_name=$(basename "$md" .md)
    
    # Sanitize filename: replace spaces with hyphens, lowercase
    safe_name=$(echo "$base_name" | tr ' ' '-' | tr '[:upper:]' '[:lower:]')
    
    out="$OUT_DIR/$safe_name.tex"
    echo "  $base_name -> $out"
    # Use --syntax-highlighting=idiomatic if available, otherwise default
    # Removed --listings, --standalone not needed
    pandoc --from=markdown --to=latex --syntax-highlighting=idiomatic "$md" -o "$out"
done

# Run python cleaner if it exists and python interpreter is provided
if [ -f "scripts/clean-unicode.py" ] && [ ! -z "$PYTHON" ]; then
    echo "Cleaning unicode in $OUT_DIR with $PYTHON..."
    "$PYTHON" scripts/clean-unicode.py "$OUT_DIR"
fi
