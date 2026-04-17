INSTALL_DIR := $(HOME)/.claude/bin
BIN_NAME := fix-cs-indent

.PHONY: build install clean test verify

build:
	cargo build --release

install: build
	@mkdir -p $(INSTALL_DIR)
	cp target/release/$(BIN_NAME) $(INSTALL_DIR)/$(BIN_NAME)
	@echo "installed: $(INSTALL_DIR)/$(BIN_NAME)"

test:
	cargo test

verify: install
	@bash scripts/verify.sh

clean:
	cargo clean
