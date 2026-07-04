#!/bin/bash
set -e

APP_NAME="ImageViewer"
VERSION="0.1.0"
BUNDLE_NAME="${APP_NAME}.app"
DMG_NAME="${APP_NAME}-${VERSION}.dmg"
VOLUME_NAME="${APP_NAME}"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Building ${APP_NAME} for macOS...${NC}"

# Build release binary
echo -e "${BLUE}Compiling release binary...${NC}"
cargo build --release

# Create app bundle structure
echo -e "${BLUE}Creating app bundle structure...${NC}"
rm -rf "${BUNDLE_NAME}"
mkdir -p "${BUNDLE_NAME}/Contents/MacOS"
mkdir -p "${BUNDLE_NAME}/Contents/Resources"

# Copy binary
echo -e "${BLUE}Copying binary...${NC}"
cp target/release/image_viewer "${BUNDLE_NAME}/Contents/MacOS/${APP_NAME}-bin"

# Create AppleScript source for the launcher
echo -e "${BLUE}Creating AppleScript launcher...${NC}"
mkdir -p "${BUNDLE_NAME}/Contents/Resources/Scripts"

cat > "${BUNDLE_NAME}/Contents/Resources/Scripts/main.scpt" << 'APPLESCRIPT_EOF'
on run
    set binPath to (POSIX path of (path to me)) & "Contents/MacOS/ImageViewer-bin"
    do shell script "'" & binPath & "' > /dev/null 2>&1 &"
end run

on open theFiles
    set binPath to (POSIX path of (path to me)) & "Contents/MacOS/ImageViewer-bin"
    repeat with aFile in theFiles
        set filePath to POSIX path of aFile
        do shell script "'" & binPath & "' '" & filePath & "' > /dev/null 2>&1 &"
        exit repeat
    end repeat
end open
APPLESCRIPT_EOF

# Compile the AppleScript
osacompile -o "${BUNDLE_NAME}/Contents/Resources/Scripts/main.scpt" "${BUNDLE_NAME}/Contents/Resources/Scripts/main.scpt"

# Create a simple launcher executable that runs the AppleScript
cat > "${BUNDLE_NAME}/Contents/MacOS/${APP_NAME}" << 'LAUNCHER_EOF'
#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
SCRIPT_PATH="${DIR}/../Resources/Scripts/main.scpt"

if [ $# -gt 0 ]; then
    # Pass files to the binary directly
    exec "${DIR}/ImageViewer-bin" "$@"
else
    # Launch without arguments
    exec "${DIR}/ImageViewer-bin"
fi
LAUNCHER_EOF

chmod +x "${BUNDLE_NAME}/Contents/MacOS/${APP_NAME}"

echo -e "${BLUE}Launcher created${NC}"

# Create Info.plist
echo -e "${BLUE}Creating Info.plist...${NC}"
cat > "${BUNDLE_NAME}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.example.imageviewer</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Image</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.image</string>
                <string>public.png</string>
                <string>public.jpeg</string>
                <string>com.compuserve.gif</string>
                <string>public.tiff</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

# Create a simple icon (optional - you can replace with actual icon later)
# For now, we'll skip icon creation

# Create temporary DMG directory
echo -e "${BLUE}Preparing DMG contents...${NC}"
TEMP_DMG_DIR=$(mktemp -d)
cp -R "${BUNDLE_NAME}" "${TEMP_DMG_DIR}/"

# Create symbolic link to Applications folder
ln -s /Applications "${TEMP_DMG_DIR}/Applications"

# Create DMG
echo -e "${BLUE}Creating DMG...${NC}"
rm -f "${DMG_NAME}"

hdiutil create -volname "${VOLUME_NAME}" \
    -srcfolder "${TEMP_DMG_DIR}" \
    -ov \
    -format UDZO \
    "${DMG_NAME}"

# Clean up
rm -rf "${TEMP_DMG_DIR}"

echo -e "${GREEN}✓ DMG created successfully: ${DMG_NAME}${NC}"
echo -e "${GREEN}✓ App bundle created: ${BUNDLE_NAME}${NC}"
echo ""
echo "To test the app:"
echo "  open ${BUNDLE_NAME}"
echo ""
echo "To mount the DMG:"
echo "  open ${DMG_NAME}"
