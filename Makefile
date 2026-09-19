KERNEL_DIR := kernel

.PHONY: build run test

build:
	cd $(KERNEL_DIR) && cargo build

run:
	cd $(KERNEL_DIR) && cargo run

test:
	cargo test