# 画像ビューア 設計書

## アーキテクチャ概要

### アーキテクチャパターン
**Elm Architecture (TEA)** を採用（icedの標準パターン）

```
┌─────────────┐
│    View     │ ← 状態を元にUIを描画
└──────┬──────┘
       │
       ↓ Message
┌─────────────┐
│   Update    │ ← メッセージを処理して状態を更新
└──────┬──────┘
       │
       ↓ State
┌─────────────┐
│    Model    │ ← アプリケーションの状態
└─────────────┘
```

## モジュール構成

```
image-viewer/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs              # エントリーポイント
│   ├── app.rs               # アプリケーションのメインロジック（Model, Message, Update, View）
│   ├── image_loader.rs      # 画像読み込み・管理
│   ├── thumbnail.rs         # サムネイル生成・キャッシュ
│   ├── file_manager.rs      # ファイルリスト管理・ソート
│   ├── zoom.rs              # ズーム機能
│   ├── config.rs            # 設定の保存・読み込み
│   └── ui/
│       ├── mod.rs
│       ├── detail_view.rs   # 詳細表示モードのUI
│       ├── thumbnail_view.rs # サムネイル一覧モードのUI
│       └── theme.rs         # カラースキーム・スタイル定義
└── tests/
    ├── image_loader_test.rs
    ├── file_manager_test.rs
    └── zoom_test.rs
```

## データモデル

### Model（アプリケーション状態）

```rust
pub struct Model {
    // ファイル管理
    current_dir: PathBuf,
    image_files: Vec<PathBuf>,
    current_index: usize,
    
    // 表示状態
    view_mode: ViewMode,
    current_image: Option<ImageHandle>,
    zoom_level: ZoomLevel,
    
    // サムネイル
    thumbnails: HashMap<PathBuf, ThumbnailHandle>,
    thumbnail_selected_index: usize,
    thumbnail_scroll_offset: f32,
    
    // 設定
    config: Config,
    
    // UI状態
    window_size: (u32, u32),
}

pub enum ViewMode {
    Detail,      // 詳細表示モード
    Thumbnail,   // サムネイル一覧モード
}

pub enum ZoomLevel {
    Fit,         // ウィンドウにフィット
    Percentage(f32), // 25%, 50%, 100%, 150%, 200%, etc.
}
```

### Message（イベント）

```rust
pub enum Message {
    // ファイル操作
    FileOpened(PathBuf),
    ImagesLoaded(Vec<PathBuf>),
    
    // ナビゲーション
    NextImage,
    PreviousImage,
    FirstImage,
    LastImage,
    JumpToImage(usize),
    
    // ズーム
    ZoomIn,
    ZoomOut,
    ZoomFit,
    ZoomActualSize,
    ZoomToLevel(f32),
    
    // 表示モード切り替え
    ToggleThumbnailView,
    
    // サムネイル操作
    ThumbnailClicked(usize),
    ThumbnailNavigate(Direction),
    ThumbnailGenerated(PathBuf, ThumbnailHandle),
    
    // 画像ロード
    ImageLoaded(ImageHandle),
    ImageLoadError(String),
    
    // ウィンドウ
    WindowResized(u32, u32),
    
    // アプリケーション
    Quit,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
```

## コアモジュール設計

### 1. file_manager.rs - ファイル管理

```rust
pub struct FileManager {
    files: Vec<PathBuf>,
    current_index: usize,
}

impl FileManager {
    pub fn new() -> Self;
    pub fn load_directory(&mut self, path: &Path) -> Result<()>;
    pub fn get_current(&self) -> Option<&PathBuf>;
    pub fn next(&mut self) -> Option<&PathBuf>;
    pub fn previous(&mut self) -> Option<&PathBuf>;
    pub fn jump_to(&mut self, index: usize) -> Option<&PathBuf>;
    pub fn current_index(&self) -> usize;
    pub fn total_count(&self) -> usize;
    fn is_supported_image(path: &Path) -> bool;
    fn sort_files(&mut self);
}
```

**責務**:
- ディレクトリ内の画像ファイル一覧を取得
- ファイル名でソート
- 現在のインデックス管理
- 次/前の画像へのナビゲーション

### 2. image_loader.rs - 画像読み込み

```rust
pub struct ImageLoader {
    cache: LruCache<PathBuf, ImageHandle>,
}

impl ImageLoader {
    pub fn new(capacity: usize) -> Self;
    pub async fn load_image(&mut self, path: &Path) -> Result<ImageHandle>;
    pub async fn preload_adjacent(&mut self, paths: &[PathBuf]);
    fn decode_image(path: &Path) -> Result<DynamicImage>;
}

pub struct ImageHandle {
    pub data: DynamicImage,
    pub width: u32,
    pub height: u32,
    pub file_size: u64,
}
```

**責務**:
- 画像ファイルのデコード
- LRUキャッシュによるメモリ管理
- 前後の画像のプリロード（非同期）
- 画像メタデータの取得

### 3. thumbnail.rs - サムネイル生成

```rust
pub struct ThumbnailManager {
    cache: HashMap<PathBuf, ThumbnailHandle>,
    thumbnail_size: u32,
    generation_queue: VecDeque<PathBuf>,
}

impl ThumbnailManager {
    pub fn new(thumbnail_size: u32) -> Self;
    pub fn get_or_generate(&mut self, path: &Path) -> Option<ThumbnailHandle>;
    pub async fn generate_thumbnail(path: &Path, size: u32) -> Result<ThumbnailHandle>;
    pub fn enqueue_generation(&mut self, paths: Vec<PathBuf>);
    pub fn clear_cache(&mut self);
}

pub struct ThumbnailHandle {
    pub handle: image::Handle,
    pub width: u32,
    pub height: u32,
}
```

**責務**:
- サムネイル画像の生成（アスペクト比維持）
- サムネイルのキャッシュ管理
- 非同期生成キュー

### 4. zoom.rs - ズーム計算

```rust
pub struct ZoomCalculator;

impl ZoomCalculator {
    pub fn calculate_fit_scale(
        image_width: u32,
        image_height: u32,
        window_width: u32,
        window_height: u32,
    ) -> f32;
    
    pub fn calculate_scaled_dimensions(
        original_width: u32,
        original_height: u32,
        zoom_level: f32,
    ) -> (u32, u32);
    
    pub fn zoom_in(current: f32) -> f32;
    pub fn zoom_out(current: f32) -> f32;
}
```

**責務**:
- ウィンドウサイズに合わせたフィット計算
- ズームレベルの段階的変更（25%刻み）
- スケーリング後のサイズ計算

### 5. config.rs - 設定管理

```rust
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub window_width: u32,
    pub window_height: u32,
    pub last_directory: Option<PathBuf>,
    pub thumbnail_size: u32,
    pub background_color: Color,
}

impl Config {
    pub fn load() -> Result<Self>;
    pub fn save(&self) -> Result<()>;
    pub fn default() -> Self;
    fn config_path() -> PathBuf; // ~/.config/image-viewer/config.toml
}
```

**責務**:
- 設定ファイルの保存・読み込み（TOML形式）
- デフォルト設定の提供
- 設定ファイルパスの決定（XDG Base Directory準拠）

## UI設計

### 1. detail_view.rs - 詳細表示

```rust
pub fn view_detail(model: &Model) -> Element<Message> {
    container(
        column![
            header_bar(model),
            image_display(model),
            footer_bar(model),
        ]
    )
    .style(theme::Container::Background)
    .into()
}

fn header_bar(model: &Model) -> Element<Message>;
fn image_display(model: &Model) -> Element<Message>;
fn footer_bar(model: &Model) -> Element<Message>;
```

**レイアウト構造**:
- ヘッダー: ファイル名、解像度、ファイルサイズ
- メイン: 画像表示（中央配置、ズーム適用）
- フッター: ページ番号、ズームレベル、ヘルプテキスト

### 2. thumbnail_view.rs - サムネイル一覧

```rust
pub fn view_thumbnails(model: &Model) -> Element<Message> {
    container(
        column![
            thumbnail_header(model),
            scrollable(thumbnail_grid(model)),
            thumbnail_footer(),
        ]
    )
    .style(theme::Container::Background)
    .into()
}

fn thumbnail_header(model: &Model) -> Element<Message>;
fn thumbnail_grid(model: &Model) -> Element<Message>;
fn thumbnail_footer() -> Element<Message>;
```

**レイアウト構造**:
- グリッド表示（Flexboxベース）
- スクロール可能
- 選択中の項目をハイライト

### 3. theme.rs - スタイル定義

```rust
pub struct Theme;

impl Theme {
    pub const BACKGROUND: Color = Color::from_rgb(0.169, 0.169, 0.169); // #2b2b2b
    pub const TEXT: Color = Color::from_rgb(0.8, 0.8, 0.8); // #cccccc
    pub const ACCENT: Color = Color::from_rgb(0.29, 0.62, 1.0); // #4a9eff
    pub const THUMBNAIL_BG: Color = Color::from_rgb(0.227, 0.227, 0.227); // #3a3a3a
}
```

## 非同期処理設計

### 画像ロード
- **iced::Command** を使用して非同期実行
- `tokio` ランタイムで画像デコード
- 結果を `Message::ImageLoaded` で通知

```rust
fn load_image(path: PathBuf) -> Command<Message> {
    Command::perform(
        async move {
            let image = image::open(&path).unwrap();
            ImageHandle::from(image)
        },
        Message::ImageLoaded,
    )
}
```

### サムネイル生成
- バックグラウンドで順次生成
- 生成完了時に `Message::ThumbnailGenerated` で通知
- 表示中の範囲を優先的に生成

## キーボードショートカット処理

```rust
fn handle_keyboard(model: &mut Model, key: keyboard::KeyCode) -> Command<Message> {
    match key {
        KeyCode::Right | KeyCode::Space => Message::NextImage,
        KeyCode::Left | KeyCode::Backspace => Message::PreviousImage,
        KeyCode::T => Message::ToggleThumbnailView,
        KeyCode::Plus | KeyCode::Equals => Message::ZoomIn,
        KeyCode::Minus => Message::ZoomOut,
        KeyCode::Key0 => Message::ZoomActualSize,
        KeyCode::F => Message::ZoomFit,
        KeyCode::Q | KeyCode::Escape => Message::Quit,
        _ => return Command::none(),
    }
}
```

## エラーハンドリング

### エラーの種類
```rust
#[derive(Debug)]
pub enum AppError {
    ImageLoadError(String),
    DirectoryReadError(String),
    ConfigLoadError(String),
    ThumbnailGenerationError(String),
}
```

### エラー表示
- UIに通知バー表示
- ログ出力
- ユーザーに分かりやすいメッセージ

## パフォーマンス最適化戦略

### メモリ管理
- **LRUキャッシュ**: 最大3画像（現在+前後1枚）
- **サムネイルキャッシュ**: 最大100枚まで保持
- 大きな画像は自動的にダウンサンプリング

### レンダリング最適化
- 画像の変更がない場合は再描画をスキップ
- サムネイルは遅延ロード（Intersection Observer的な実装）

### 非同期処理
- 画像デコード: バックグラウンドスレッド
- サムネイル生成: 非同期キュー
- プリロード: 次の画像を先読み

## テスト戦略

### 単体テスト
- `file_manager`: ファイルソート、インデックス操作
- `zoom`: スケール計算
- `config`: 設定の保存・読み込み

### 統合テスト
- 画像ロード → 表示 → ナビゲーション
- サムネイル生成 → キャッシュ
- ズーム → 画像スケーリング

### 手動テスト項目
- [ ] 各種画像フォーマットの表示
- [ ] 大きな画像（10MB+）の表示
- [ ] 大量の画像（1000枚+）のサムネイル生成
- [ ] キーボードショートカットの動作
- [ ] ウィンドウリサイズ時の挙動

## ビルド・デプロイ

### 開発環境
```bash
cargo run -- /path/to/image.jpg
```

### リリースビルド
```bash
cargo build --release
```

### macOS アプリバンドル
- `cargo-bundle` を使用してアプリバンドル作成
- アイコン作成（.icns）
- Info.plist設定（関連付ける拡張子）

## 依存関係

```toml
[dependencies]
iced = "0.12"
image = { version = "0.25", features = ["jpeg", "png", "gif", "bmp", "webp", "tiff"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
lru = "0.12"
```

## マイルストーン

### Phase 1: 基本機能（MVP）
- [ ] 画像読み込み・表示
- [ ] キーボードナビゲーション
- [ ] 基本的なズーム機能

### Phase 2: サムネイル機能
- [ ] サムネイル生成
- [ ] グリッド表示
- [ ] サムネイルクリックで詳細表示

### Phase 3: 最適化・UX改善
- [ ] プリロード実装
- [ ] 設定の永続化
- [ ] エラーハンドリング

### Phase 4: リリース準備
- [ ] テスト完了
- [ ] ドキュメント整備
- [ ] macOSアプリバンドル作成
