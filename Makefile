VST3_DIR = $(HOME)/.vst3
CLAP_DIR = $(HOME)/.clap
BIN_DIR  = $(HOME)/.local/bin

.PHONY: build install clean uninstall

build:
	cargo xtask bundle fretscope --release

install: build
	@mkdir -p $(VST3_DIR) $(CLAP_DIR) $(BIN_DIR)
	cp -r target/bundled/fretscope.vst3 $(VST3_DIR)/
	cp target/bundled/fretscope.clap $(CLAP_DIR)/
	cp target/bundled/fretscope $(BIN_DIR)/
	@echo ""
	@echo "Installed:"
	@echo "  VST3       -> $(VST3_DIR)/fretscope.vst3"
	@echo "  CLAP       -> $(CLAP_DIR)/fretscope.clap"
	@echo "  Standalone -> $(BIN_DIR)/fretscope"

uninstall:
	rm -rf $(VST3_DIR)/fretscope.vst3
	rm -f $(CLAP_DIR)/fretscope.clap
	rm -f $(BIN_DIR)/fretscope

clean:
	cargo clean
