# =============================================================================
# JITOS Root Makefile - Convenience wrapper for docs/tex/Makefile
# =============================================================================

TEX_DIR = docs/tex

.PHONY: all clean arch computer adrs rfcs whitepaper master help

# Default target - show help
help:
	@echo "JITOS Documentation Build System"
	@echo "================================="
	@echo ""
	@echo "Available targets:"
	@echo "  make all         - Build all PDFs (arch, computer, adrs, rfcs, whitepaper, master)"
	@echo "  make arch        - Build ARCHITECTURE.pdf"
	@echo "  make computer    - Build COMPUTER.pdf"
	@echo "  make adrs        - Build ADRs.pdf"
	@echo "  make rfcs        - Build RFC.pdf"
	@echo "  make whitepaper  - Build WHITEPAPER.pdf"
	@echo "  make master      - Build JITOS_COMPLETE.pdf (all books combined)"
	@echo "  make clean       - Remove all build artifacts and PDFs"
	@echo ""
	@echo "PDFs will be created in the root directory."

# Build targets - delegate to docs/tex/Makefile
all:
	@$(MAKE) -C $(TEX_DIR) all

arch:
	@$(MAKE) -C $(TEX_DIR) arch

computer:
	@$(MAKE) -C $(TEX_DIR) computer

adrs:
	@$(MAKE) -C $(TEX_DIR) adrs

rfcs:
	@$(MAKE) -C $(TEX_DIR) rfcs

whitepaper:
	@$(MAKE) -C $(TEX_DIR) whitepaper

master:
	@$(MAKE) -C $(TEX_DIR) master

clean:
	@$(MAKE) -C $(TEX_DIR) clean
	@echo "All build artifacts cleaned."
