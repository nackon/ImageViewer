APP_NAME := ImageViewer
BUNDLE_DIR := src-tauri/target/release/bundle
DEBUG_BUNDLE_DIR := src-tauri/target/debug/bundle
APP_BUNDLE := $(BUNDLE_DIR)/macos/$(APP_NAME).app
DMG_FILE = $(firstword $(wildcard $(BUNDLE_DIR)/dmg/*.dmg))

.PHONY: help setup install-deps build release run test fmt clippy dmg open-app open-dmg clean generate-test-images

help:
	@echo "利用可能なコマンド:"
	@echo "  make setup                 フロントエンド依存関係をインストール (npm ci)"
	@echo "  make install-deps          開発依存関係のチェックとインストール"
	@echo "  make build                 デバッグビルド"
	@echo "  make release               リリースビルド"
	@echo "  make run                   開発モードでアプリを実行"
	@echo "  make test                  テストを実行 (Rust + JS)"
	@echo "  make fmt                   コードフォーマットチェック"
	@echo "  make clippy                Clippyによる静的解析"
	@echo "  make dmg                   DMGパッケージを作成"
	@echo "  make open-app              .appバンドルを開く"
	@echo "  make open-dmg              DMGファイルをマウント"
	@echo "  make clean                 ビルド成果物をクリーン"
	@echo "  make generate-test-images  テスト用画像を生成（generate_test_images バイナリが必要）"

setup:
	npm ci

install-deps:
	@command -v cargo >/dev/null 2>&1 || { echo "Rust (cargo) が見つかりません。https://rustup.rs/ からインストールしてください"; exit 1; }
	@command -v npm >/dev/null 2>&1 || { echo "Node.js (npm) が見つかりません。https://nodejs.org/ からインストールしてください"; exit 1; }
	npm install

build:
	npm run tauri -- build --debug

release:
	npm run tauri -- build

run:
	npm run tauri:dev

test:
	cd src-tauri && cargo test --verbose
	npm test

fmt:
	cd src-tauri && cargo fmt -- --check

clippy:
	cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings

dmg: release
	@if [ -n "$(DMG_FILE)" ]; then \
		echo "DMGを作成しました: $(DMG_FILE)"; \
	else \
		echo "DMGファイルが見つかりません: $(BUNDLE_DIR)/dmg"; \
		exit 1; \
	fi

open-app:
	@if [ -d "$(APP_BUNDLE)" ]; then \
		open "$(APP_BUNDLE)"; \
	else \
		echo ".appバンドルが見つかりません。先に 'make release' を実行してください: $(APP_BUNDLE)"; \
		exit 1; \
	fi

open-dmg:
	@if [ -n "$(DMG_FILE)" ]; then \
		open "$(DMG_FILE)"; \
	else \
		echo "DMGファイルが見つかりません。先に 'make dmg' を実行してください: $(BUNDLE_DIR)/dmg"; \
		exit 1; \
	fi

clean:
	cd src-tauri && cargo clean
	rm -rf dist

generate-test-images:
	@if [ -x ./generate_test_images ]; then \
		./generate_test_images; \
	else \
		echo "generate_test_images バイナリが見つかりません（生成ツールのソースはこのリポジトリに含まれていません）"; \
		exit 1; \
	fi

