.PHONY: help build run test clean dmg release

APP_NAME = ImageViewer
VERSION = 0.1.0

help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build debug version
	cargo build

release: ## Build release version
	cargo build --release

run: ## Run the application
	cargo run

test: ## Run tests
	cargo test

clean: ## Clean build artifacts
	cargo clean
	rm -rf $(APP_NAME).app
	rm -f $(APP_NAME)-*.dmg
	rm -f generate_test_images

dmg: ## Create macOS DMG package
	@chmod +x create-dmg.sh
	./create-dmg.sh

open-app: ## Open the app bundle
	open $(APP_NAME).app

open-dmg: ## Open the DMG file
	open $(APP_NAME)-$(VERSION).dmg

generate-test-images: ## Generate test images
	rustc --edition 2021 generate_test_images.rs -L target/debug/deps --extern image=$$(find target/debug/deps -name "libimage-*.rlib" | head -1)
	./generate_test_images

install-deps: ## Install development dependencies
	@echo "Checking for Rust..."
	@command -v cargo >/dev/null 2>&1 || (echo "Please install Rust from https://rustup.rs/" && exit 1)
	@echo "✓ Rust is installed"

.DEFAULT_GOAL := help
