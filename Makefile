.PHONY: help build run mint clean

help:
	@echo "Available commands:"
	@echo "  make build   - Build the project"
	@echo "  make run     - Run the main binary"
	@echo "  make mint    - Run the mint binary"
	@echo "  make clean   - Clean build artifacts"

build:
	cargo build

run:
	cargo run

mint:
	cargo run --bin mint

create:
	cargo run --bin create

clean:
	cargo clean
