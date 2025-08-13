// Memory optimization module
// Provides Arc<str> interning for commonly used strings and object pooling

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Global string interner for commonly used configuration strings
static STRING_INTERNER: OnceLock<Mutex<StringInterner>> = OnceLock::new();

/// String interner for reducing memory allocation of commonly used strings
#[derive(Debug, Default)]
pub struct StringInterner {
    strings: HashMap<String, Arc<str>>,
    stats: InternerStats,
}

/// Statistics for monitoring string interner performance
#[derive(Debug, Default, Clone)]
pub struct InternerStats {
    pub total_requests: usize,
    pub cache_hits: usize,
    pub unique_strings: usize,
    pub memory_saved_bytes: usize,
}

impl StringInterner {
    /// Get or create an Arc<str> for the given string
    pub fn intern(&mut self, s: &str) -> Arc<str> {
        self.stats.total_requests += 1;
        
        if let Some(existing) = self.strings.get(s) {
            self.stats.cache_hits += 1;
            self.stats.memory_saved_bytes += s.len();
            existing.clone()
        } else {
            let arc_str: Arc<str> = Arc::from(s);
            self.strings.insert(s.to_string(), arc_str.clone());
            self.stats.unique_strings += 1;
            arc_str
        }
    }
    
    /// Get statistics about the interner
    pub fn stats(&self) -> &InternerStats {
        &self.stats
    }
    
    /// Clear the interner (useful for testing or memory cleanup)
    pub fn clear(&mut self) {
        self.strings.clear();
        self.stats = InternerStats::default();
    }
    
    /// Get the number of interned strings
    pub fn len(&self) -> usize {
        self.strings.len()
    }
    
    /// Check if the interner is empty
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
}

/// Global function to intern strings
pub fn intern_string(s: &str) -> Arc<str> {
    let interner = STRING_INTERNER.get_or_init(|| Mutex::new(StringInterner::default()));
    let mut interner = interner.lock().unwrap();
    interner.intern(s)
}

/// Get interner statistics
pub fn get_interner_stats() -> InternerStats {
    if let Some(interner) = STRING_INTERNER.get() {
        if let Ok(interner) = interner.lock() {
            return interner.stats().clone();
        }
    }
    InternerStats::default()
}

/// Common Hyprland configuration strings that are frequently used
pub struct CommonStrings {
    pub general: Arc<str>,
    pub input: Arc<str>,
    pub decoration: Arc<str>,
    pub animations: Arc<str>,
    pub dwindle: Arc<str>,
    pub master: Arc<str>,
    pub misc: Arc<str>,
    pub bind: Arc<str>,
    pub exec: Arc<str>,
    pub hyprctl: Arc<str>,
    pub windowrule: Arc<str>,
    pub layerrule: Arc<str>,
    pub workspace: Arc<str>,
    pub monitor: Arc<str>,
    pub plugin: Arc<str>,
    pub source: Arc<str>,
    pub env: Arc<str>,
    pub keyword: Arc<str>,
}

impl CommonStrings {
    /// Create a new instance with all common strings interned
    pub fn new() -> Self {
        Self {
            general: intern_string("general"),
            input: intern_string("input"),
            decoration: intern_string("decoration"),
            animations: intern_string("animations"),
            dwindle: intern_string("dwindle"),
            master: intern_string("master"),
            misc: intern_string("misc"),
            bind: intern_string("bind"),
            exec: intern_string("exec"),
            hyprctl: intern_string("hyprctl"),
            windowrule: intern_string("windowrule"),
            layerrule: intern_string("layerrule"),
            workspace: intern_string("workspace"),
            monitor: intern_string("monitor"),
            plugin: intern_string("plugin"),
            source: intern_string("source"),
            env: intern_string("env"),
            keyword: intern_string("keyword"),
        }
    }
}

impl Default for CommonStrings {
    fn default() -> Self {
        Self::new()
    }
}

/// Object pool for reusing expensive objects
pub struct ObjectPool<T> {
    objects: Arc<Mutex<Vec<T>>>,
    create_fn: Arc<dyn Fn() -> T + Send + Sync>,
    reset_fn: Option<Arc<dyn Fn(&mut T) + Send + Sync>>,
    max_size: usize,
    stats: Arc<Mutex<PoolStats>>,
}

/// Statistics for monitoring object pool performance
#[derive(Debug, Default, Clone)]
pub struct PoolStats {
    pub total_gets: usize,
    pub pool_hits: usize,
    pub objects_created: usize,
    pub objects_returned: usize,
    pub current_pool_size: usize,
    pub max_pool_size: usize,
}

impl<T> ObjectPool<T> {
    /// Create a new object pool
    pub fn new<F>(create_fn: F, max_size: usize) -> Self 
    where 
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            objects: Arc::new(Mutex::new(Vec::new())),
            create_fn: Arc::new(create_fn),
            reset_fn: None,
            max_size,
            stats: Arc::new(Mutex::new(PoolStats { max_pool_size: max_size, ..Default::default() })),
        }
    }
    
    /// Create a new object pool with a reset function
    pub fn with_reset<F, R>(create_fn: F, reset_fn: R, max_size: usize) -> Self 
    where 
        F: Fn() -> T + Send + Sync + 'static,
        R: Fn(&mut T) + Send + Sync + 'static,
    {
        Self {
            objects: Arc::new(Mutex::new(Vec::new())),
            create_fn: Arc::new(create_fn),
            reset_fn: Some(Arc::new(reset_fn)),
            max_size,
            stats: Arc::new(Mutex::new(PoolStats { max_pool_size: max_size, ..Default::default() })),
        }
    }
    
    /// Get an object from the pool or create a new one
    pub fn get(&self) -> PooledObject<T> {
        let mut objects = self.objects.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        stats.total_gets += 1;
        
        if let Some(mut obj) = objects.pop() {
            stats.pool_hits += 1;
            stats.current_pool_size = objects.len();
            
            // Reset the object if a reset function was provided
            if let Some(ref reset_fn) = self.reset_fn {
                reset_fn(&mut obj);
            }
            
            drop(objects);
            drop(stats);
            
            PooledObject {
                object: Some(obj),
                pool: self.objects.clone(),
                stats: self.stats.clone(),
                max_size: self.max_size,
            }
        } else {
            stats.objects_created += 1;
            drop(objects);
            drop(stats);
            
            let obj = (self.create_fn)();
            PooledObject {
                object: Some(obj),
                pool: self.objects.clone(),
                stats: self.stats.clone(),
                max_size: self.max_size,
            }
        }
    }
    
    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Clear the pool
    pub fn clear(&self) {
        let mut objects = self.objects.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        objects.clear();
        stats.current_pool_size = 0;
    }
}

/// A pooled object that returns itself to the pool when dropped
pub struct PooledObject<T> {
    object: Option<T>,
    pool: Arc<Mutex<Vec<T>>>,
    stats: Arc<Mutex<PoolStats>>,
    max_size: usize,
}

impl<T> PooledObject<T> {
    /// Get a reference to the pooled object
    pub fn as_ref(&self) -> &T {
        self.object.as_ref().unwrap()
    }
    
    /// Get a mutable reference to the pooled object
    pub fn as_mut(&mut self) -> &mut T {
        self.object.as_mut().unwrap()
    }
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.object.take() {
            let mut pool = self.pool.lock().unwrap();
            let mut stats = self.stats.lock().unwrap();
            
            stats.objects_returned += 1;
            
            if pool.len() < self.max_size {
                pool.push(obj);
                stats.current_pool_size = pool.len();
            }
            // If pool is full, the object is simply dropped
        }
    }
}

/// Common object pools for the application
pub struct CommonPools {
    pub string_vec_pool: ObjectPool<Vec<String>>,
    pub hashmap_pool: ObjectPool<HashMap<String, String>>,
}

impl CommonPools {
    /// Create common object pools with reasonable defaults
    pub fn new() -> Self {
        Self {
            string_vec_pool: ObjectPool::with_reset(
                Vec::new,
                |v: &mut Vec<String>| v.clear(),
                50, // Pool size
            ),
            hashmap_pool: ObjectPool::with_reset(
                HashMap::new,
                |m: &mut HashMap<String, String>| m.clear(),
                25, // Pool size
            ),
        }
    }
}

impl Default for CommonPools {
    fn default() -> Self {
        Self::new()
    }
}

/// Global common pools instance
static COMMON_POOLS: OnceLock<CommonPools> = OnceLock::new();

/// Get the global common pools
pub fn get_common_pools() -> &'static CommonPools {
    COMMON_POOLS.get_or_init(CommonPools::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interning() {
        let mut interner = StringInterner::default();
        
        let s1 = interner.intern("test");
        let s2 = interner.intern("test");
        let s3 = interner.intern("other");
        
        // Same string should return the same Arc
        assert!(Arc::ptr_eq(&s1, &s2));
        
        // Different strings should have different Arcs
        assert!(!Arc::ptr_eq(&s1, &s3));
        
        // Stats should be correct
        let stats = interner.stats();
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.unique_strings, 2);
    }
    
    #[test]
    fn test_object_pool() {
        let pool = ObjectPool::new(|| Vec::<i32>::new(), 2);
        
        let mut obj1 = pool.get();
        obj1.push(1);
        obj1.push(2);
        
        let stats_before = pool.stats();
        assert_eq!(stats_before.objects_created, 1);
        assert_eq!(stats_before.pool_hits, 0);
        
        drop(obj1); // Return to pool
        
        let obj2 = pool.get(); // Should get from pool
        let stats_after = pool.stats();
        
        assert_eq!(stats_after.pool_hits, 1);
        // Without reset function, the vector will still contain the values
        assert_eq!(obj2.len(), 2);
    }
    
    #[test]
    fn test_object_pool_with_reset() {
        let pool = ObjectPool::with_reset(
            || Vec::<i32>::new(),
            |v: &mut Vec<i32>| v.clear(),
            2
        );
        
        let mut obj1 = pool.get();
        obj1.push(1);
        obj1.push(2);
        assert_eq!(obj1.len(), 2);
        
        drop(obj1);
        
        let obj2 = pool.get();
        assert_eq!(obj2.len(), 0); // Should be cleared by reset function
    }
    
    #[test]
    fn test_common_strings() {
        let common = CommonStrings::new();
        
        // Should be able to access all common strings
        assert_eq!(common.general.as_ref(), "general");
        assert_eq!(common.input.as_ref(), "input");
        assert_eq!(common.decoration.as_ref(), "decoration");
        
        // Creating another instance should reuse the same interned strings
        let common2 = CommonStrings::new();
        assert!(Arc::ptr_eq(&common.general, &common2.general));
    }
}