PREFIX ?= $(HOME)/.local
BINDIR = $(PREFIX)/bin
HYPRCONF = $(HOME)/.config/hypr/hyprland.conf

.PHONY: deps build install uninstall

deps:
ifeq ($(shell command -v pacman 2>/dev/null),)
ifeq ($(shell command -v zypper 2>/dev/null),)
	$(error No supported package manager found. Install manually: gtk4, cairo, tesseract, hyprshot, wl-clipboard)
else
	sudo zypper install -y gtk4-devel cairo-devel tesseract-ocr tesseract-ocr-traineddata-english hyprshot wl-clipboard
endif
else
	sudo pacman -S --needed --noconfirm gtk4 cairo tesseract tesseract-data-eng wl-clipboard
	@command -v hyprshot >/dev/null 2>&1 || { echo "hyprshot not found — install from AUR: yay -S hyprshot"; exit 1; }
endif

build:
	cargo build --release

install: deps build
	mkdir -p $(BINDIR)
	cp target/release/hyprsnap $(BINDIR)/hyprsnap
	@for script in screenshot-edit ocr-select; do \
		src="$$script.sh"; \
		dst="$(BINDIR)/$$script"; \
		if [ -f "$$dst" ]; then \
			if ! diff -q "$$src" "$$dst" >/dev/null 2>&1; then \
				printf "$$dst already exists and has been modified. Overwrite? [y/N] "; \
				read ans; \
				case "$$ans" in [yY]*) cp "$$src" "$$dst" ;; *) echo "Skipping $$dst" ;; esac; \
			else \
				cp "$$src" "$$dst"; \
			fi; \
		else \
			cp "$$src" "$$dst"; \
		fi; \
	done
	chmod +x $(BINDIR)/screenshot-edit $(BINDIR)/ocr-select
	@echo ""
	@echo "Installed to $(BINDIR)."
	@echo ""
	@echo "Add the following to your Hyprland config ($(HYPRCONF)):"
	@echo ""
	@echo "  # Screenshots (region select -> annotation editor)"
	@echo "  bind = \$$mod SHIFT, S, exec, ~/.local/bin/screenshot-edit"
	@echo ""
	@echo "  # OCR (select region -> text to clipboard)"
	@echo "  bind = \$$mod SHIFT, O, exec, ~/.local/bin/ocr-select"
	@echo ""
	@echo "Then reload: hyprctl reload"

uninstall:
	rm -f $(BINDIR)/hyprsnap
	rm -f $(BINDIR)/screenshot-edit
	rm -f $(BINDIR)/ocr-select
	@echo "Removed binaries. Clean up keybindings in $(HYPRCONF) manually."
