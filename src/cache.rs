use std::sync::Arc;

use cached::Cached;
use cached::SizedCache;
use lru::LruCache;
use parking_lot::Mutex;
use quick_cache::sync::Cache as QuickCache;

pub trait Cache {
    type Item: Clone;

    fn get_key(&self, key: &usize) -> Option<Self::Item>;

    fn set_key(&self, key: usize, value: Self::Item);
}

impl<T: Clone> Cache for Arc<QuickCache<usize, T>> {
    type Item = T;

    fn get_key(&self, key: &usize) -> Option<Self::Item> {
        self.get(key)
    }

    fn set_key(&self, key: usize, value: Self::Item) {
        self.insert(key, value);
    }
}

impl<T: Clone> Cache for Arc<Mutex<LruCache<usize, T>>> {
    type Item = T;

    fn get_key(&self, key: &usize) -> Option<Self::Item> {
        self.lock().get(key).cloned()
    }

    fn set_key(&self, key: usize, value: Self::Item) {
        self.lock().put(key, value);
    }
}

impl<T: Clone> Cache for Arc<Mutex<SizedCache<usize, T>>> {
    type Item = T;

    fn get_key(&self, key: &usize) -> Option<Self::Item> {
        self.lock().cache_get(key).cloned()
    }

    fn set_key(&self, key: usize, value: Self::Item) {
        self.lock().cache_set(key, value);
    }
}
