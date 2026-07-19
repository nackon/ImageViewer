# ImageViewer - Tauri Prototype

Simple image viewer built with Rust + Tauri.

This is a prototype to evaluate Tauri as an alternative to Iced for building native desktop apps.

## 特徴

- 🚀 高速な画像切り替え（200ms以内）
- ⌨️ キーボード操作に最適化
- 🔍 スムーズなズーム機能
- 🖼️ サムネイル一覧表示
- 💾 LRUキャッシュによる効率的なメモリ管理
- 📁 フォルダ内の画像を自動読み込み

## 対応フォーマット

- JPEG/JPG
- PNG
- GIF（静止画）
- BMP
- WebP
- TIFF

## 必要環境

- macOS 10.13 以降
- Rust 1.70+ (開発時)

## インストール

### 方法1: DMGからインストール（推奨）

1. 最新のDMGファイルをダウンロード
2. DMGをマウント
3. `ImageViewer.app` を `Applications` フォルダにドラッグ&ドロップ

### 方法2: ソースからビルド

```bash
# リポジトリをクローン
git clone <repository-url>
cd ImageViewer

# 依存関係をインストール（Rustが必要）
make install-deps

# リリースビルド
make release

# アプリを実行
make run
```

## 使い方

### 起動方法

1. **ファイルをダブルクリック**: 画像ファイルを `ImageViewer.app` で開く
2. **ドラッグ&ドロップ**: 画像を `ImageViewer.app` にドロップ
3. **コマンドライン**: `./ImageViewer.app/Contents/MacOS/ImageViewer /path/to/image.jpg`

画像を開くと、同じフォルダ内の全画像が自動的に読み込まれます。

### キーボード操作

#### 画像ナビゲーション
| キー | 動作 |
|------|------|
| `→` / `Space` / `N` | 次の画像 |
| `←` / `Backspace` / `P` | 前の画像 |
| `Home` | 最初の画像 |
| `End` | 最後の画像 |

#### ズーム操作
| キー | 動作 |
|------|------|
| `+` / `=` | ズームイン（25%刻み） |
| `-` | ズームアウト（25%刻み） |
| `0` | 実サイズ（100%） |
| `W` | フィット表示（ウィンドウに合わせる） |
| マウスホイール | ズームイン/アウト |

#### その他
| キー | 動作 |
|------|------|
| `F` | 全画面表示切り替え |
| `T` | サムネイル一覧表示切り替え |
| `Q` / `Esc` | アプリ終了（全画面中の `Esc` は全画面解除） |

### サムネイル一覧モード

`T` キーでサムネイル一覧表示に切り替えます。

- `↑` `↓` `←` `→` キーで選択移動
- `Enter` で選択した画像を詳細表示
- クリックで直接選択
- `T` または `Esc` で一覧を閉じる

## ビルド方法

### 開発版を実行

```bash
npm run tauri:dev
# または
make run
```

### リリースビルド

```bash
npm run tauri -- build
# または
make release
```

ビルド成果物は `src-tauri/target/release/bundle/macos/ImageViewer.app` に生成されます。

### macOS DMG作成

```bash
make dmg
```

`src-tauri/target/release/bundle/dmg/` にDMGファイルが生成されます。

**注**: DMGにはアプリケーションアイコンが含まれています。アイコンは `assets/icon/` ディレクトリに配置されています。

### 便利なMakeコマンド

```bash
make help                  # 利用可能なコマンドを表示
make build                 # デバッグビルド
make release               # リリースビルド
make run                   # アプリを実行
make test                  # テストを実行
make dmg                   # DMGパッケージを作成
make open-app              # .appバンドルを開く
make open-dmg              # DMGファイルをマウント
make clean                 # ビルド成果物をクリーン
make generate-test-images  # テスト用画像を生成
make install-deps          # 開発依存関係のチェック
```

## テスト

### テストの実行

```bash
cargo test
# または
make test
```

### テスト用画像の生成

開発・テスト用に、ダミー画像を簡単に生成できます:

```bash
make generate-test-images
```

`test_images/` ディレクトリに100個のテスト画像（PNG/JPEG混在）が生成されます。
各画像は異なるパターン（グラデーション、チェッカーボード等）で800x600pxのサイズです。

## アーキテクチャ

### モジュール構成

```
src/
├── main.rs           # エントリポイント
├── app.rs            # メインアプリケーションロジック
├── file_manager.rs   # ファイル管理
├── image_loader.rs   # 非同期画像読み込み
├── image_cache.rs    # LRUキャッシュ
├── zoom.rs           # ズーム機能
└── ui/               # UI コンポーネント
    ├── mod.rs
    └── thumbnail.rs  # サムネイル表示
```

### 主な機能

- **非同期画像読み込み**: Tokioによる非同期処理でUIをブロックしない
- **LRUキャッシュ**: 前後の画像をプリロードし、メモリ効率を保つ
- **サムネイル生成**: 遅延ロードとキャッシュで高速表示
- **icedフレームワーク**: 宣言的UIによる保守性の高い設計

## トラブルシューティング

### アプリが起動しない

macOSのセキュリティ設定で、未署名のアプリがブロックされている可能性があります:

1. `システム設定` → `プライバシーとセキュリティ` を開く
2. "開く許可" で `ImageViewer.app` を許可

または、右クリック → "開く" で初回起動してください。

### 画像が表示されない

- サポートされているフォーマットか確認してください
- ファイルが破損していないか確認してください
- コンソールログを確認: `cargo run` で起動するとエラーメッセージが表示されます

### メモリ使用量が多い

- LRUキャッシュは自動的に古い画像を削除しますが、大きな画像を連続して開くと一時的にメモリを多く使用します
- `src/image_cache.rs` のキャッシュサイズを調整できます（デフォルト: 5枚）

## ライセンス

このプロジェクトのライセンスについては [LICENSE](LICENSE) を参照してください。

## 今後の予定

- [ ] スライドショー機能
- [ ] 画像の簡易編集（回転、トリミング）
- [ ] プレイリスト/お気に入り機能
- [x] フルスクリーンモード
- [ ] EXIF情報表示
- [ ] RAW画像対応
- [ ] アニメーションGIF対応
- [ ] Windows/Linux対応

## 貢献

バグレポートや機能リクエストは Issue でお願いします。
プルリクエストも歓迎します！
