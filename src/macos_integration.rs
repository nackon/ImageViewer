#[cfg(target_os = "macos")]
pub mod macos_integration {
    use std::path::PathBuf;
    use iced::futures::{channel::mpsc, stream::Stream};
    use objc2::{rc::Retained, runtime::{AnyObject, AnyClass, Sel}, sel};
    use objc2_app_kit::NSApplication;
    use foundation::{NSArray, NSString};

    // グローバルな通知用ストリームの送信元
    static mut SENDER_CHANNEL: Option<mpsc::UnboundedSender<PathBuf>> = None;

    // winit内部クラスに直接埋め込むためのファイルオープン通知メソッド (C互換の生関数)
    extern "C" fn swizzled_open_files(
        _this: *mut AnyObject,
        _sel: Sel,
        _sender: *mut AnyObject,
        filenames: *mut AnyObject, // OSから渡されるファイル名の配列ポインタ
    ) {
        eprintln!("[DEBUG] swizzled_open_files が呼ばれました");
        unsafe {
            if let Some(ref tx) = SENDER_CHANNEL {
                eprintln!("[DEBUG] SENDER_CHANNELが初期化されています");
                // 安全に AnyObject の参照から NSArray の型へと型安全に紐付ける
                if !filenames.is_null() {
                    let array_ref = &*(filenames as *const NSArray<NSString>);

                    // 配列の要素数を取得してループを回す
                    let count = array_ref.count();
                    eprintln!("[DEBUG] ファイル数: {}", count);
                    for i in 0..count {
                        let ns_str = array_ref.objectAtIndex(i);
                        let path_str = ns_str.to_string();
                        let path = PathBuf::from(path_str);
                        eprintln!("[DEBUG] ファイルパス: {:?}", path);

                        // iced 側のストリームに送信
                        match tx.unbounded_send(path) {
                            Ok(_) => eprintln!("[DEBUG] メッセージ送信成功"),
                            Err(e) => eprintln!("[DEBUG] メッセージ送信失敗: {:?}", e),
                        }
                    }
                } else {
                    eprintln!("[DEBUG] filenames が null です");
                }
            } else {
                eprintln!("[DEBUG] SENDER_CHANNELがまだ初期化されていません");
            }
        }
    }

    // 起動直後（main関数の1行目）に最速で呼び出す関数
    pub fn pre_init_macos_listener() {
        // winit が内部的に使用しているクラス「WinitAppDelegate」を取得
        if let Some(winit_class) = AnyClass::get("WinitAppDelegate") {
            let selector = sel!(application:openFiles:);
            let types = "v@:@@\0"; // メソッドの型シグネチャ

            unsafe {
                let _ = objc2::ffi::class_addMethod(
                    winit_class as *const AnyClass as *mut _,
                    selector.as_ptr(),
                    Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(swizzled_open_files as *const ())),
                    types.as_ptr() as *const _,
                );
            }
        }
    }

    // iced の Subscription 用のストリームを流す
    pub fn listen_open_file_events() -> impl Stream<Item = PathBuf> {
        let (tx, rx) = mpsc::unbounded::<PathBuf>();
        unsafe {
            SENDER_CHANNEL = Some(tx);
        }
        rx
    }
}
