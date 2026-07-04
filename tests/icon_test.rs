use std::path::Path;

#[test]
fn test_icon_files_exist() {
    // 必須のアイコンファイルが存在することを確認
    let icon_dir = Path::new("assets/icon");
    assert!(icon_dir.exists(), "Icon directory should exist");

    // 各サイズのPNGファイルが存在することを確認
    let sizes = vec![16, 32, 64, 128, 256, 512, 1024];
    for size in sizes {
        let icon_path = icon_dir.join(format!("icon_{0}x{0}.png", size));
        assert!(
            icon_path.exists(),
            "Icon file {} should exist",
            icon_path.display()
        );
    }

    // メインアイコンファイル
    let main_icon = icon_dir.join("icon.png");
    assert!(main_icon.exists(), "Main icon.png should exist");

    // macOS用のicnsファイル
    let icns_file = icon_dir.join("icon.icns");
    assert!(icns_file.exists(), "icon.icns should exist");
}

#[test]
fn test_icon_files_are_valid_png() {
    use std::fs::File;
    use std::io::Read;

    let icon_dir = Path::new("assets/icon");
    let sizes = vec![16, 32, 64, 128, 256, 512, 1024];

    for size in sizes {
        let icon_path = icon_dir.join(format!("icon_{0}x{0}.png", size));
        if !icon_path.exists() {
            continue; // スキップ（前のテストで失敗する）
        }

        // PNGファイルのマジックナンバーを確認
        let mut file = File::open(&icon_path).expect("Failed to open icon file");
        let mut header = [0u8; 8];
        file.read_exact(&mut header)
            .expect("Failed to read PNG header");

        // PNGのマジックナンバー: 89 50 4E 47 0D 0A 1A 0A
        let png_signature: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(
            header,
            png_signature,
            "Icon file {} should be a valid PNG file",
            icon_path.display()
        );
    }
}

#[test]
fn test_icns_file_is_valid() {
    use std::fs::File;
    use std::io::Read;

    let icns_path = Path::new("assets/icon/icon.icns");
    if !icns_path.exists() {
        return; // icnsファイルが存在しない場合はスキップ（macOSのみ）
    }

    // ICNSファイルのマジックナンバーを確認
    let mut file = File::open(icns_path).expect("Failed to open icns file");
    let mut header = [0u8; 4];
    file.read_exact(&mut header)
        .expect("Failed to read ICNS header");

    // ICNSのマジックナンバー: "icns" (0x69 0x63 0x6E 0x73)
    let icns_signature: [u8; 4] = [0x69, 0x63, 0x6E, 0x73];
    assert_eq!(
        header, icns_signature,
        "icon.icns should be a valid ICNS file"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn test_icon_is_referenced_in_info_plist() {
    // Info.plistにアイコンが正しく参照されているか確認
    // これはビルド後のテストなので、create-dmg.shの実行が必要
    // ここでは、create-dmg.shスクリプトにCFBundleIconFileが含まれているか確認

    use std::fs;

    let create_dmg_script =
        fs::read_to_string("create-dmg.sh").expect("Failed to read create-dmg.sh");

    assert!(
        create_dmg_script.contains("CFBundleIconFile"),
        "create-dmg.sh should set CFBundleIconFile in Info.plist"
    );

    assert!(
        create_dmg_script.contains("AppIcon"),
        "create-dmg.sh should reference AppIcon"
    );

    assert!(
        create_dmg_script.contains("icon.icns"),
        "create-dmg.sh should copy icon.icns file"
    );
}
