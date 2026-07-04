#[cfg(target_os = "macos")]
pub mod macos_integration {
    use std::path::PathBuf;
    use iced::futures::{channel::mpsc, stream::Stream};
    use objc2::{runtime::{AnyClass, Sel}, sel};
    use foundation::{NSArray, NSString};

    // グローバルな通知用ストリームの送信元
    static mut SENDER_CHANNEL: Option<mpsc::UnboundedSender<PathBuf>> = None;

    // winit内部クラスに直接埋め込むためのファイルオープン通知メソッド (C互換の生関数)
    extern "C" fn swizzled_open_files(
        _this: *mut objc2::runtime::AnyObject,
        _sel: Sel,
        _sender: *mut objc2::runtime::AnyObject,
        filenames: *mut objc2::runtime::AnyObject,
    ) {
        unsafe {
            if let Some(ref tx) = SENDER_CHANNEL {
                // 生ポインタから安全に NSArray<NSString> の参照を復元
                let filenames_ref: &NSArray<NSString> = &*(filenames as *const NSArray<NSString>);
                for filename in filenames_ref {
                    let path_str = filename.to_string();
                    let path = PathBuf::from(path_str);
                    let _ = tx.unbounded_send(path);
                }
            }
        }
    }

    // 起動直後（main関数の1行目）に最速で呼び出す関数
    pub fn pre_init_macos_listener() {
        // winit が内部的に使用しているクラス「WinitAppDelegate」を取得
        if let Some(winit_class) = AnyClass::get("WinitAppDelegate") {
            let selector = sel!(application:openFiles:);
            let types = "v@:@@\0"; // メソッドの型シグネチャ（void, self, selector, id, id）

            unsafe {
                // ★ 正しいパスである objc2::ffi::class_addMethod を使用します
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
