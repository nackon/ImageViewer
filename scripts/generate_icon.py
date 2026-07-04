#!/usr/bin/env python3
"""
ImageViewer アプリケーションアイコン生成スクリプト

カラフルな写真ギャラリースタイルのアイコンを複数サイズで生成し、
macOS用の.icnsファイルを作成します。

使用方法:
    python3 scripts/generate_icon.py

必要なパッケージ:
    pip install Pillow
"""

from PIL import Image, ImageDraw
import os
import subprocess
import sys

def create_gallery_icon(size):
    """カラフルな写真ギャラリースタイルアイコンを生成"""
    img = Image.new('RGBA', (size, size), (255, 255, 255, 0))
    draw = ImageDraw.Draw(img)

    # 背景円
    center = size // 2
    radius = size * 0.45
    draw.ellipse([center-radius, center-radius, center+radius, center+radius],
                 fill=(50, 50, 60, 255))

    # 複数の重なった写真フレーム
    colors = [
        (255, 100, 100),  # 赤
        (100, 200, 255),  # 青
        (100, 255, 150),  # 緑
    ]

    angles = [-15, 0, 15]
    for i, (color, angle) in enumerate(zip(colors, angles)):
        # フレームサイズ
        frame_size = int(size * 0.39)
        frame_x = center - frame_size // 2
        frame_y = center - frame_size // 2 - int(size * 0.04) + i * int(size * 0.02)

        # 回転用の一時イメージ
        temp = Image.new('RGBA', (size, size), (255, 255, 255, 0))
        temp_draw = ImageDraw.Draw(temp)

        # 白いフレーム
        border = int(size * 0.02)
        temp_draw.rounded_rectangle([frame_x-border, frame_y-border,
                                    frame_x+frame_size+border, frame_y+frame_size+border],
                                   radius=int(size * 0.02), fill=(255, 255, 255, 255))

        # カラー画像部分
        temp_draw.rounded_rectangle([frame_x, frame_y,
                                    frame_x+frame_size, frame_y+frame_size],
                                   radius=int(size * 0.01), fill=color)

        # 回転して合成
        rotated = temp.rotate(angle, center=(center, center), resample=Image.BICUBIC)
        img = Image.alpha_composite(img, rotated)

    return img

def generate_icons():
    """全サイズのアイコンを生成"""
    # 出力ディレクトリ
    output_dir = "assets/icon"
    os.makedirs(output_dir, exist_ok=True)

    # macOS用の各サイズを生成
    sizes = [16, 32, 64, 128, 256, 512, 1024]

    print("Generating icon files...")
    for size in sizes:
        icon = create_gallery_icon(size)
        filename = f"{output_dir}/icon_{size}x{size}.png"
        icon.save(filename)
        print(f"  ✓ Created: {filename}")

    # メインアイコン（512px）
    icon_512 = create_gallery_icon(512)
    icon_512.save(f"{output_dir}/icon.png")
    print(f"  ✓ Created: {output_dir}/icon.png")

    return output_dir

def create_icns(output_dir):
    """macOS用の.icnsファイルを作成"""
    print("\nCreating macOS .icns file...")

    iconset_dir = f"{output_dir}/AppIcon.iconset"
    os.makedirs(iconset_dir, exist_ok=True)

    # iconset用のファイル名マッピング
    icon_mappings = [
        ("icon_16x16.png", "icon_16x16.png"),
        ("icon_32x32.png", "icon_16x16@2x.png"),
        ("icon_32x32.png", "icon_32x32.png"),
        ("icon_64x64.png", "icon_32x32@2x.png"),
        ("icon_128x128.png", "icon_128x128.png"),
        ("icon_256x256.png", "icon_128x128@2x.png"),
        ("icon_256x256.png", "icon_256x256.png"),
        ("icon_512x512.png", "icon_256x256@2x.png"),
        ("icon_512x512.png", "icon_512x512.png"),
        ("icon_1024x1024.png", "icon_512x512@2x.png"),
    ]

    # iconset用にコピー
    for src, dst in icon_mappings:
        src_path = f"{output_dir}/{src}"
        dst_path = f"{iconset_dir}/{dst}"
        if os.path.exists(src_path):
            with Image.open(src_path) as img:
                img.save(dst_path)

    # icnsファイルを生成（macOSのみ）
    try:
        icns_path = f"{output_dir}/icon.icns"
        subprocess.run([
            "iconutil", "-c", "icns", iconset_dir, "-o", icns_path
        ], check=True, capture_output=True)
        print(f"  ✓ Created: {icns_path}")
    except subprocess.CalledProcessError as e:
        print(f"  ✗ Failed to create .icns file: {e}")
        print("  Note: iconutil is only available on macOS")
        return False
    except FileNotFoundError:
        print("  ✗ iconutil not found")
        print("  Note: iconutil is only available on macOS")
        return False

    return True

def main():
    """メイン処理"""
    print("ImageViewer Icon Generator")
    print("=" * 50)

    try:
        from PIL import Image
    except ImportError:
        print("Error: Pillow is not installed")
        print("Please install it with: pip install Pillow")
        sys.exit(1)

    # アイコン生成
    output_dir = generate_icons()

    # .icnsファイル作成
    create_icns(output_dir)

    print("\n" + "=" * 50)
    print("Icon generation complete!")
    print(f"Icons saved to: {output_dir}/")

if __name__ == "__main__":
    main()
