INSTALL_DIR := $(HOME)/.claude/bin
BIN_NAME := fix-cs-indent
CARGO_TARGET_DIR ?= target
TARGET_BIN := $(CARGO_TARGET_DIR)/release/$(BIN_NAME)

export CARGO_TARGET_DIR

.PHONY: build clean install test verify

build:
	cargo build --release

test:
	cargo test

verify: build test
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	@bash scripts/verify.sh

install: verify
	@mkdir -p "$(INSTALL_DIR)"
	cp "$(TARGET_BIN)" "$(INSTALL_DIR)/$(BIN_NAME)"
	@echo "installed: $(INSTALL_DIR)/$(BIN_NAME)"

clean:
	cargo clean
