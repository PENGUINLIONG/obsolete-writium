use std::sync::{Arc, Mutex, RwLock};

type CacheGeneration<T, Extra> = fn(&Extra, &str) -> Option<T>;
type CacheDisposal<T, Extra> = fn(&Extra, &str, &mut T);
/// Cache for each WritiumApi. Any WritiumApi can be composited with this struct
/// for cache.
pub struct Cache<T, Extra> {
    extra: Extra,
    max_size: usize,
    cache: Mutex<Vec<(String, Arc<RwLock<T>>)>>,
    generate: CacheGeneration<T, Extra>,
    dispose: CacheDisposal<T, Extra>,
}
impl<T, Extra> Cache<T, Extra> {
    pub fn new(
        max_size: usize,
        gen: CacheGeneration<T, Extra>,
        dis: CacheDisposal<T, Extra>,
        extra: Extra,
    ) -> Cache<T, Extra> {
        Cache {
            extra: extra,
            max_size: max_size,
            cache: Mutex::new(Vec::new()),
            generate: gen,
            dispose: dis,
        }
    }

    /// Get the object identified by given ID. If the object is not cached, try
    /// generating its cache with provided generation function. If there is no
    /// space for another object, the .
    pub fn get(&self, id: &str) -> Option<Arc<RwLock<T>>> {
        // Check if cache exists.
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(pos) = cache.iter()
                .position(|x| x.0 == id) {
                // Update disposal priority.
                let intermediate = cache.remove(pos);
                cache.push(intermediate);
                return Some(cache[pos].1.clone())
            }
        }
        // Doesn't exist in cache. Generate one.
        let new_obj = match (self.generate)(&self.extra, id) {
            Some(obj) => obj,
            None => return None,
        };
        {
            let mut cache = self.cache.lock().unwrap();
            // Reached maximum size, dispose one.
            if cache.len() == self.max_size {
                (self.dispose)(
                    &self.extra,
                    &cache.first().unwrap().0,
                    // Cache write is locked, so there won't be anything
                    // accessing the object to be disposed after we get the
                    // access to the object.
                    &mut *cache.first().unwrap().1.write().unwrap(),
                );
                cache.remove(0);
            }
            // Register the fresh new object.
            cache.push((
                id.to_string(),
                Arc::new(RwLock::new(new_obj))
            ));
        }
        Some(self.cache.lock().unwrap().last().unwrap().1.clone())
    }
}
