#[cfg(target_os = "macos")]
pub mod macos_integration {
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use iced::futures::{channel::mpsc, stream::Stream};
    use objc2::{declare_class, mutability, rc::Retained, ClassType, DeclaredClass};
    use foundation::{NSNotification, NSNotificationCenter, NSString};

    const NS_APPLICATION_OPEN_FILE_NOTIFICATION: &str = "NSApplicationOpenFileNotification";

    static SENDER: OnceLock<Mutex<mpsc::UnboundedSender<PathBuf>>> = OnceLock::new();

    declare_class!(
        struct NotificationObserver;

        unsafe impl ClassType for NotificationObserver {
            type Super = objc2::runtime::NSObject;
            type Mutability = mutability::InteriorMutable;
            const NAME: &'static str = "NotificationObserver";
        }

        impl DeclaredClass for NotificationObserver {
            type Ivars = ();
        }

        unsafe impl NotificationObserver {
            #[method(onOpenFile:)]
            fn on_open_file(&self, notification: &NSNotification) {
                if let Some(user_info) = unsafe { notification.userInfo() } {
                    let key = NSString::from_str("NSDevicePath");
                    unsafe {
                        if let Some(path_nsstr) = user_info.objectForKey(&key) {
                            let raw_ptr: *const objc2::runtime::AnyObject = Retained::as_ptr(&path_nsstr);
                            let ns_str: &NSString = &*(raw_ptr as *const NSString);
                            let path_str = ns_str.to_string();
                            let path = PathBuf::from(path_str);

                            if let Some(sender) = SENDER.get() {
                                if let Ok(guard) = sender.lock() {
                                    let _ = guard.unbounded_send(path);
                                }
                            }
                        }
                    }
                }
            }
        }
    );

    pub fn listen_open_file_events() -> impl Stream<Item = PathBuf> {
        let (tx, rx) = mpsc::unbounded::<PathBuf>();

        SENDER.set(Mutex::new(tx)).ok();

        let observer: Retained<NotificationObserver> = unsafe {
            objc2::msg_send_id![NotificationObserver::alloc(), init]
        };

        let center = unsafe { NSNotificationCenter::defaultCenter() };
        let notification_name = NSString::from_str(NS_APPLICATION_OPEN_FILE_NOTIFICATION);

        unsafe {
            center.addObserver_selector_name_object(
                &observer,
                objc2::sel!(onOpenFile:),
                Some(&notification_name),
                None,
            );
        }

        std::mem::forget(observer);

        rx
    }
}
