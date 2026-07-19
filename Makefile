.PHONY: setup test fmt clippy build

setup:
	npm ci

test:
	cd src-tauri && cargo test --verbose

fmt:
	cd src-tauri && cargo fmt -- --check

clippy:
	cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings

build:
	npm run build
