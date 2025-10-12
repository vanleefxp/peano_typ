CURRENT_DIR := $(dir $(MAKEFILE_LIST))
COMPILE_TARGET := wasm32-wasip1
LIB_NAME := math_utils

build:
	@cd $(CURRENT_DIR)
	cargo fmt
# 	cargo clippy --all-targets --all-features -- -D warnings
	cargo build --release --target $(COMPILE_TARGET)

copy:
	@cd $(CURRENT_DIR)
	wasi-stub "./target/$(COMPILE_TARGET)/release/$(LIB_NAME).wasm" -o "../../src/_impl/$(subst _,-,$(LIB_NAME)).wasm"