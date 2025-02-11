use std::sync::Arc;

use cached::Cached;
use cached::SizedCache;
use lru::LruCache;
use parking_lot::Mutex;
use quick_cache::sync::Cache as QuickCache;

pub trait Cache {
    fn get_key(&self, key: &usize) -> Option<String>;
    fn set_key(&self, key: usize, value: String);
}

impl Cache for Arc<QuickCache<usize, String>> {
    fn get_key(&self, key: &usize) -> Option<String> {
        self.get(key)
    }

    fn set_key(&self, key: usize, value: String) {
        self.insert(key, value);
    }
}

impl Cache for Arc<Mutex<LruCache<usize, String>>> {
    fn get_key(&self, key: &usize) -> Option<String> {
        self.lock().get(key).cloned()
    }

    fn set_key(&self, key: usize, value: String) {
        self.lock().put(key, value);
    }
}

impl Cache for Arc<Mutex<SizedCache<usize, String>>> {
    fn get_key(&self, key: &usize) -> Option<String> {
        self.lock().cache_get(key).cloned()
    }

    fn set_key(&self, key: usize, value: String) {
        self.lock().cache_set(key, value);
    }
}
