use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thread_priority::ThreadPriority;

use crate::image_loader::ImageData;

pub struct ImageCache {
    cache: Arc<Mutex<LruCache<PathBuf, ImageData>>>,
    cache_size: usize,
}

impl ImageCache {
    pub fn new(cache_size: usize) -> Self {
        let capacity = NonZeroUsize::new(cache_size).unwrap_or(NonZeroUsize::new(10).unwrap());
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            cache_size,
        }
    }

    pub fn get(&self, path: &PathBuf) -> Option<ImageData> {
        let mut cache = self.cache.lock().ok()?;
        cache.get(path).cloned()
    }

    pub fn put(&self, path: PathBuf, image: ImageData) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(path, image);
        }
    }

    pub fn prefetch(&self, paths: Vec<PathBuf>) {
        let cache = self.cache.clone();
        std::thread::spawn(move || {
            let _ = thread_priority::set_current_thread_priority(ThreadPriority::Min);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                for path in paths {
                    let needs_load = {
                        let cache_guard = cache.lock().ok();
                        cache_guard.map_or(true, |c| !c.contains(&path))
                    };

                    if needs_load {
                        let path_clone = path.clone();
                        let cache_clone = cache.clone();

                        if let Ok(image_data) = ImageData::load(&path_clone) {
                            if let Ok(mut cache_guard) = cache_clone.lock() {
                                cache_guard.put(image_data.path.clone(), image_data);
                            }
                        }
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            });
        });
    }

    #[allow(dead_code)]
    pub fn cache_size(&self) -> usize {
        self.cache_size
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new(10)
    }
}

impl Clone for ImageCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            cache_size: self.cache_size,
        }
    }
}
