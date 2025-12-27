# =============================================================================
# JITOS Documentation Build System
# Builds both DARK MODE (Dracula) and PRINT (Vanilla) versions
# =============================================================================

# Directories
BUILD_DIR = build
CHAPTERS_DIR = $(BUILD_DIR)/chapters
SOURCES_DIR = sources
BOOKS_DIR = books
SCRIPTS_DIR = scripts
VENV_DIR = $(SCRIPTS_DIR)/../venv
PYTHON = $(VENV_DIR)/bin/python3

# Source Directories (Relative to docs/tex/ for md conversion)
SRC_MD_ARCH = ../../docs/ARCH
SRC_MD_COMPUTER = ../../docs/COMPUTER
SRC_MD_ADR = ../../docs/ADR
SRC_MD_RFC = ../../docs/RFC
SRC_MD_WHITEPAPER = ../../docs/WHITEPAPER.md

# Tools
CONVERT_MD_TO_TEX = $(SCRIPTS_DIR)/convert.sh
CLEAN_UNICODE = $(PYTHON) $(SCRIPTS_DIR)/clean-unicode.py
LATEX = xelatex
LATEX_FLAGS = -interaction=nonstopmode -halt-on-error -shell-escape -output-directory=$(BUILD_DIR)

# Color output
COLOR_RESET = \033[0m
COLOR_BOLD = \033[1m
COLOR_GREEN = \033[32m
COLOR_BLUE = \033[34m
COLOR_PURPLE = \033[35m

# =============================================================================
# Main Targets
# =============================================================================

.PHONY: all dark print template clean init help sotu-2025 sotu-2025-dark

help:
	@echo "$(COLOR_BOLD)JITOS Documentation Build System$(COLOR_RESET)"
	@echo "======================================"
	@echo ""
	@echo "$(COLOR_PURPLE)Main Targets:$(COLOR_RESET)"
	@echo "  make all    - Build ALL 14 PDFs (dark + regular versions)"
	@echo "  make dark   - Build 7 dark mode PDFs (Dracula theme, -DARK suffix)"
	@echo "  make print  - Build 7 regular PDFs (vanilla, no suffix)"
	@echo "  make clean  - Remove all build artifacts"
	@echo ""
	@echo "$(COLOR_PURPLE)Individual Dark Mode PDFs (-DARK suffix):$(COLOR_RESET)"
	@echo "  make arch-dark computer-dark adrs-dark rfcs-dark whitepaper-dark aion-holography-dark master-dark"
	@echo ""
	@echo "$(COLOR_PURPLE)Individual Regular PDFs (no suffix):$(COLOR_RESET)"
	@echo "  make arch computer adrs rfcs whitepaper aion-holography master"
	@echo ""
	@echo "$(COLOR_PURPLE)Utility:$(COLOR_RESET)"
	@echo "  make template - Build docs/tex/template.tex to build/template.pdf"
	@echo ""
	@echo "$(COLOR_PURPLE)Reports:$(COLOR_RESET)"
	@echo "  make sotu-2025       - Build SOTU-2025.pdf"
	@echo "  make sotu-2025-dark  - Build SOTU-2025-DARK.pdf"

all: dark print

dark: arch-dark computer-dark adrs-dark rfcs-dark whitepaper-dark aion-holography-dark master-dark
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)✓ All 7 dark mode PDFs complete$(COLOR_RESET)"

print: arch computer adrs rfcs whitepaper aion-holography master
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)✓ All 7 regular PDFs complete$(COLOR_RESET)"

template: init
	@echo "$(COLOR_BOLD)$(COLOR_BLUE)Building template.tex...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) template.tex
	@$(LATEX) $(LATEX_FLAGS) template.tex > /dev/null
	@echo "$(COLOR_BOLD)$(COLOR_BLUE)✓ build/template.pdf created$(COLOR_RESET)"

init:
	@mkdir -p $(BUILD_DIR)

# =============================================================================
# DARK MODE Targets
# =============================================================================

arch-dark: init
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)Building ARCHITECTURE (Dark Mode)...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/arch-dark.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/arch-dark.tex > /dev/null
	@cp $(BUILD_DIR)/arch-dark.pdf ../../ARCHITECTURE-DARK.pdf
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)✓ ARCHITECTURE-DARK.pdf created$(COLOR_RESET)"

computer-dark: init
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)Building CΩMPUTER (Dark Mode)...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/computer-dark.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/computer-dark.tex > /dev/null
	@cp $(BUILD_DIR)/computer-dark.pdf ../../COMPUTER-DARK.pdf
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)✓ COMPUTER-DARK.pdf created$(COLOR_RESET)"

adrs-dark: init
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)Building ADRs (Dark Mode)...$(COLOR_RESET)"
	@ls $(SOURCES_DIR)/adr/*.tex | sort | sed 's|.*/||;s|^|\\input{sources/adr/|;s|$$|}|' > $(BUILD_DIR)/adrs_manifest.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/adrs-dark.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/adrs-dark.tex > /dev/null
	@cp $(BUILD_DIR)/adrs-dark.pdf ../../ADRs-DARK.pdf
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)✓ ADRs-DARK.pdf created$(COLOR_RESET)"

rfcs-dark: init
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)Building RFCs (Dark Mode)...$(COLOR_RESET)"
	@ls $(SOURCES_DIR)/rfc/*.tex | sort | sed 's|.*/||;s|^|\\input{sources/rfc/|;s|$$|}|' > $(BUILD_DIR)/rfcs_manifest.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/rfcs-dark.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/rfcs-dark.tex > /dev/null
	@cp $(BUILD_DIR)/rfcs-dark.pdf ../../RFC-DARK.pdf
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)✓ RFC-DARK.pdf created$(COLOR_RESET)"

whitepaper-dark: init
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)Building Whitepaper (Dark Mode)...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/whitepaper-dark.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/whitepaper-dark.tex > /dev/null
	@cp $(BUILD_DIR)/whitepaper-dark.pdf ../../WHITEPAPER-DARK.pdf
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)✓ WHITEPAPER-DARK.pdf created$(COLOR_RESET)"

aion-holography-dark: init
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)Building AIΩN Holography (Dark Mode)...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/aion-holography-dark.tex
	@bibtex $(BUILD_DIR)/aion-holography-dark
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/aion-holography-dark.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/aion-holography-dark.tex > /dev/null
	@cp $(BUILD_DIR)/aion-holography-dark.pdf ../../AION-HOLOGRAPHY-DARK.pdf
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)✓ AION-HOLOGRAPHY-DARK.pdf created$(COLOR_RESET)"

master-dark: arch-dark computer-dark adrs-dark rfcs-dark whitepaper-dark aion-holography-dark
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)Building JITOS Complete (Dark Mode)...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/master-dark.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/master-dark.tex > /dev/null
	@cp $(BUILD_DIR)/master-dark.pdf ../../JITOS_COMPLETE-DARK.pdf
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)✓ JITOS_COMPLETE-DARK.pdf created$(COLOR_RESET)"

# =============================================================================
# REGULAR Targets (no suffix)
# =============================================================================

arch: init
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)Building ARCHITECTURE...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/arch-print.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/arch-print.tex > /dev/null
	@cp $(BUILD_DIR)/arch-print.pdf ../../ARCHITECTURE.pdf
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)✓ ARCHITECTURE.pdf created$(COLOR_RESET)"

computer: init
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)Building CΩMPUTER...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/computer-print.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/computer-print.tex > /dev/null
	@cp $(BUILD_DIR)/computer-print.pdf ../../COMPUTER.pdf
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)✓ COMPUTER.pdf created$(COLOR_RESET)"

adrs: init
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)Building ADRs...$(COLOR_RESET)"
	@ls $(SOURCES_DIR)/adr/*.tex | sort | sed 's|.*/||;s|^|\\input{sources/adr/|;s|$$|}|' > $(BUILD_DIR)/adrs_manifest.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/adrs-print.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/adrs-print.tex > /dev/null
	@cp $(BUILD_DIR)/adrs-print.pdf ../../ADRs.pdf
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)✓ ADRs.pdf created$(COLOR_RESET)"

rfcs: init
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)Building RFCs...$(COLOR_RESET)"
	@ls $(SOURCES_DIR)/rfc/*.tex | sort | sed 's|.*/||;s|^|\\input{sources/rfc/|;s|$$|}|' > $(BUILD_DIR)/rfcs_manifest.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/rfcs-print.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/rfcs-print.tex > /dev/null
	@cp $(BUILD_DIR)/rfcs-print.pdf ../../RFC.pdf
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)✓ RFC.pdf created$(COLOR_RESET)"

whitepaper: init
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)Building Whitepaper...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/whitepaper-print.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/whitepaper-print.tex > /dev/null
	@cp $(BUILD_DIR)/whitepaper-print.pdf ../../WHITEPAPER.pdf
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)✓ WHITEPAPER.pdf created$(COLOR_RESET)"

aion-holography: init
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)Building AIΩN Holography...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/aion-holography-print.tex
	@bibtex $(BUILD_DIR)/aion-holography-print
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/aion-holography-print.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/aion-holography-print.tex > /dev/null
	@cp $(BUILD_DIR)/aion-holography-print.pdf ../../AION-HOLOGRAPHY.pdf
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)✓ AION-HOLOGRAPHY.pdf created$(COLOR_RESET)"

master: arch computer adrs rfcs whitepaper aion-holography
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)Building JITOS Complete...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/master-print.tex
	@$(LATEX) $(LATEX_FLAGS) $(BOOKS_DIR)/master-print.tex > /dev/null
	@cp $(BUILD_DIR)/master-print.pdf ../../JITOS_COMPLETE.pdf
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)✓ JITOS_COMPLETE.pdf created$(COLOR_RESET)"

# =============================================================================
# Reports
# =============================================================================

sotu-2025: init
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)Building SOTU-2025...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) ../REPORTS/SOTU-2025-expanded.tex
	@$(LATEX) $(LATEX_FLAGS) ../REPORTS/SOTU-2025-expanded.tex > /dev/null
	@cp $(BUILD_DIR)/SOTU-2025-expanded.pdf ../../SOTU-2025.pdf
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)✓ SOTU-2025.pdf created$(COLOR_RESET)"

sotu-2025-dark: init
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)Building SOTU-2025 (Dark Mode)...$(COLOR_RESET)"
	@$(LATEX) $(LATEX_FLAGS) ../REPORTS/SOTU-2025-expanded-dark.tex
	@$(LATEX) $(LATEX_FLAGS) ../REPORTS/SOTU-2025-expanded-dark.tex > /dev/null
	@cp $(BUILD_DIR)/SOTU-2025-expanded-dark.pdf ../../SOTU-2025-DARK.pdf
	@echo "$(COLOR_BOLD)$(COLOR_PURPLE)✓ SOTU-2025-DARK.pdf created$(COLOR_RESET)"

# =============================================================================
# Cleanup
# =============================================================================

clean:
	rm -rf $(BUILD_DIR)
	rm -f ../../*-DARK.pdf
	rm -f ../../ARCHITECTURE.pdf ../../COMPUTER.pdf ../../ADRs.pdf ../../RFC.pdf ../../WHITEPAPER.pdf ../../AION-HOLOGRAPHY.pdf ../../JITOS_COMPLETE.pdf ../../SOTU-2025.pdf
	@echo "$(COLOR_BOLD)$(COLOR_GREEN)✓ All build artifacts removed$(COLOR_RESET)"
