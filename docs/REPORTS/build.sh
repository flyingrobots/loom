#!/usr/bin/env bash
# Build script for SOTU reports with proper LaTeX environment setup

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Add tex/styles directory to LaTeX search path
export TEXINPUTS="../tex/styles//:"

# Build print edition
echo "Building SOTU-2025-print.pdf..."
pdflatex -interaction=nonstopmode SOTU-2025-print.tex
pdflatex -interaction=nonstopmode SOTU-2025-print.tex  # Second pass for references

# Build dark edition
echo "Building SOTU-2025-dark.pdf..."
pdflatex -interaction=nonstopmode SOTU-2025-dark.tex
pdflatex -interaction=nonstopmode SOTU-2025-dark.tex  # Second pass for references

echo "Build complete!"
echo "  - SOTU-2025-print.pdf"
echo "  - SOTU-2025-dark.pdf"
