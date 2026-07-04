#!/bin/bash

# エラーが発生したら停止
set -e

APP_NAME="ImageViewer"
VERSION="0.1.0"
DMG_NAME="${APP_NAME}-${VERSION}.dmg"

echo "Building release binary..."
cargo build --release

# 一度ビルド先を綺麗にする
rm -rf "target/release/${APP_NAME}.app"
rm -rf "target/release/dmg_staging"
rm -f "target/release/tmp.dmg"
rm -f "${DMG_NAME}"

# ディレクトリ構造の作成
APP_DIR="target/release/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

echo "Creating .app bundle structure..."
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

echo "Copying binary and resources..."
cp "target/release/image_viewer" "${MACOS_DIR}/"

# PkgInfoの作成
echo -n "APPL????" > "${CONTENTS_DIR}/PkgInfo"

# 'EOF' で囲むことでXMLテキストをそのまま出力
echo "Creating Info.plist..."
cat << 'EOF' > "target/release/ImageViewer.app/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://apple.com">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>image_viewer</string>
    <key>CFBundleIdentifier</key>
    <string>com.nackon.imageviewer</string>
    <key>CFBundleName</key>
    <string>ImageViewer</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>

    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Supported Images</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
            <key>NSDocumentClass</key>
            <string>NSDocument</string>

            <key>CFBundleTypeExtensions</key>
            <array>
                <string>jpg</string>
                <string>jpeg</string>
                <string>png</string>
                <string>webp</string>
                <string>bmp</string>
                <string>tiff</string>
                <string>gif</string>
            </array>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.image</string>
                <string>public.jpeg</string>
                <string>public.png</string>
                <string>org.webmproject.webp</string>
                <string>com.microsoft.bmp</string>
                <string>public.tiff</string>
                <string>com.compuserve.gif</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

# --- インストーラー形式（DMG）ビルド処理の修正版 ---
echo "Preparing Installer DMG Staging..."
STAGING_DIR="target/release/dmg_staging"
mkdir -p "${STAGING_DIR}"

# パスが崩れないよう、一度 target/release に移動してシンプルな相対パスで処理する
cd target/release

# ステージング環境へ .app とショートカットを配置
cp -R "${APP_NAME}.app" "dmg_staging/"
ln -s /Applications "dmg_staging/Applications"

echo "Running hdiutil create..."
# カレントディレクトリからの相対パスで確実に作成
hdiutil create -volname "${APP_NAME} Installer" -srcfolder "dmg_staging" -ov -format UDRW "tmp.dmg"

echo "Running hdiutil convert..."
# カレントディレクトリからの相対パスで確実に変換
cd ../..
hdiutil convert "target/release/tmp.dmg" -format UDZO -o "${DMG_NAME}"

# クリーンアップ
rm -f "target/release/tmp.dmg"
rm -rf "target/release/dmg_staging"

echo "Successfully created ${DMG_NAME} (Installer format)"
