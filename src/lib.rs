use std::marker::PhantomData;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};


pub struct Cache<T> {
    begin_secs: u64,
    expires_in_secs: Duration,
    calculation: Box<dyn Fn() -> T>,
    value: Option<T>,
}
impl<T> Cache<T> {
    pub fn new(expires_in_secs: u64, calculation: impl Fn() -> T + 'static) -> Self {
        Cache {
            begin_secs: 0,
            expires_in_secs: Duration::new(expires_in_secs, 0),
            calculation: Box::new(calculation),
            value: None
        }
    }
    pub fn is_value_expires(&mut self) -> bool {
        let current_secs = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
        eprintln!("=={}=={}==", self.begin_secs + self.expires_in_secs.as_secs(), current_secs);
        if current_secs >= self.begin_secs + self.expires_in_secs.as_secs() {
            true
        } else {
            false
        }
    }
    pub fn value(&mut self) -> &T {
        self.value.get_or_insert_with(|| {
            let v = (self.calculation)();
            self.begin_secs = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
            v
        })
    }
    pub fn expires_in(&self) -> u64 {
        self.expires_in_secs.as_secs()
    }
}

pub struct Cacher<T> {
    cache_map: HashMap<String, Cache<T>>,
    _mark: PhantomData<T>,
}

impl<T> Cacher<T> {
    pub fn new() -> Self {
        Self {
            cache_map: HashMap::new(),
            _mark: PhantomData,
        }
    }
    pub fn fetch(&mut self, key: &str, expires_in_secs: u64, calculation: impl Fn() -> T + 'static) -> &T {
        let cache: Cache<T> = Cache::new(expires_in_secs, calculation);
        if self.cache_map.get(key).is_none() {
            self.cache_map.insert(key.to_string(), cache);
            let in_cache = self.cache_map.get_mut(key).unwrap();
            return in_cache.value()
        } else {
            let in_cache = self.cache_map.get_mut(key).unwrap();
            if in_cache.is_value_expires() {
                *in_cache = cache
            }
            return in_cache.value()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    #[test]
    fn test_cache() {
        let mut i32_cache = Cache::new(10, || 5);
        assert_eq!(i32_cache.value(), &5);
    }

    #[test]
    fn test_cacher() {
        let mut i32_cacher = Cacher::<i32>::new();
        {
            let v1 = i32_cacher.fetch("v1", 10, || 1);
            assert_eq!(v1, &1);
        }
        {
            let v2 = i32_cacher.fetch("v2", 10, || 2);
            assert_eq!(v2, &2);
        }
        // again
        {
            let v1 = i32_cacher.fetch("v1", 10, || 11);
            assert_eq!(v1, &1);
        }
        {
            let v2 = i32_cacher.fetch("v2", 10, || 22);
            assert_eq!(v2, &2);
        }
        //  expires
        {
            let v1 = i32_cacher.fetch("v1_expires", 3, || 1);
            assert_eq!(v1, &1);
            let v1 = i32_cacher.fetch("v1_expires", 0, || 2);
            assert_eq!(v1, &1); // 数据未过期，继续使用旧数据
            let three_secs = time::Duration::from_secs(3);
            thread::sleep(three_secs);
            let v1 = i32_cacher.fetch("v1_expires", 0, || 2); // 0 立即失效
            assert_eq!(v1, &2);
            let v1 = i32_cacher.fetch("v1_expires", 0, || 3);
            assert_eq!(v1, &3);
        }
        let mut string_cacher = Cacher::<String>::new();
        {
            let v1 = string_cacher.fetch("v1", 10, || "1".to_string());
            assert_eq!(v1, "1");
        }
        {
            let v2 = string_cacher.fetch("v2", 10, || "2".to_string());
            assert_eq!(v2, "2");
        }
        // again
        {
            let v1 = string_cacher.fetch("v1", 10, || "11".to_string());
            assert_eq!(v1, "1");
        }
        {
            let v2 = string_cacher.fetch("v2", 10, || "22".to_string());
            assert_eq!(v2, "2");
        }
    }
}