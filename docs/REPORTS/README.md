# JITOS SOTU Reports

This directory contains the State of the Union (SOTU) reports for JITOS.

## Building

Use the provided build script:

```bash
cd docs/REPORTS
./build.sh
```

This will generate:
- `SOTU-2025-print.pdf` - Print edition with standard fonts
- `SOTU-2025-dark.pdf` - Dark edition with Dracula theme

## Directory Structure

- `SOTU-2025-print.tex` - Print edition LaTeX source
- `SOTU-2025-dark.tex` - Dark edition LaTeX source
- `SOTU-2025-content.tex` - Shared content (imported by both editions)
- `build.sh` - Build script with proper LaTeX environment setup

## LaTeX Environment

The build script sets `TEXINPUTS=../tex/styles//:` to ensure LaTeX can find
the shared style files in `docs/tex/styles/`:
- `jitos-dark.sty` - Dark theme styling
- `jitos-print.sty` - Print theme styling
- `draculatheme.sty` - Dracula color definitions

## Manual Building

If you need to build manually:

```bash
cd docs/REPORTS
export TEXINPUTS="../tex/styles//:"
pdflatex -interaction=nonstopmode SOTU-2025-print.tex
pdflatex -interaction=nonstopmode SOTU-2025-print.tex  # Second pass for refs
```

The `//` in `TEXINPUTS` enables recursive directory search.
