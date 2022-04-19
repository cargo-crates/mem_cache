mod cache;

use std::marker::PhantomData;
use std::collections::HashMap;

use cache::Cache;

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
        if self.cache_map.get(key).is_none() {
            let cache: Cache<T> = Cache::new(expires_in_secs, calculation);
            self.cache_map.insert(key.to_string(), cache);
            let in_cache = self.cache_map.get_mut(key).unwrap();
            return in_cache.value()
        } else {
            let in_cache = self.cache_map.get_mut(key).unwrap();
            if in_cache.is_value_expires() {
                let cache: Cache<T> = Cache::new(expires_in_secs, calculation);
                *in_cache = cache
            }
            return in_cache.value()
        }
    }
    pub fn force_fetch(&mut self, key: &str, expires_in_secs: u64, calculation: impl Fn() -> T + 'static) -> &T {
        let cache: Cache<T> = Cache::new(expires_in_secs, calculation);
        self.cache_map.insert(key.to_string(), cache);
        self.cache_map.get_mut(key).unwrap().value()
    }
    pub fn read(&mut self, key: &str) -> anyhow::Result<&T> {
        match self.cache_map.get_mut(key) {
            Some(cache) => {
                Ok(cache.value())
            },
            None => Err(anyhow::anyhow!("cache not exists"))
        }
    }
    pub fn write(&mut self, key: &str, value: T) -> anyhow::Result<()> {
        match self.cache_map.get_mut(key) {
            Some(cache) => {
                cache.update_value(value);
                Ok(())
            },
            None => Err(anyhow::anyhow!("cache not exists"))
        }
    }
    pub fn expire(&mut self, key: &str) -> anyhow::Result<()> {
        match self.cache_map.get_mut(key) {
            Some(cache) => {
                cache.expire_value();
                Ok(())
            },
            None => Err(anyhow::anyhow!("cache not exists"))
        }
    }
    pub fn delete(&mut self, key: &str) -> Option<Cache<T>> {
        self.cache_map.remove(key)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    #[test]
    fn test_fetch() {
        let mut i32_cacher = Cacher::<i32>::new();
        {
            let a =  0;
            let v1 = i32_cacher.fetch("v1", 10, move || 1 + a);
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
            let v1 = i32_cacher.fetch("v1_expires", 0, || 3); // 0 立即失效
            assert_eq!(v1, &3);
            let v1 = i32_cacher.fetch("v1_expires", 0, || 4);
            assert_eq!(v1, &4);
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

    #[test]
    fn test_force_fetch() {
        let mut i32_cacher = Cacher::<i32>::new();
        let a =  0;
        let v1 = i32_cacher.force_fetch("v1", 10, move || 1 + a);
        assert_eq!(v1, &1);
        let v1 = i32_cacher.fetch("v1", 10, move || 3 + a);
        assert_eq!(v1, &1);
        let v1 = i32_cacher.force_fetch("v1", 10, move || 2 + a);
        assert_eq!(v1, &2);
        let v1 = i32_cacher.fetch("v1", 10, move || 3 + a);
        assert_eq!(v1, &2);
    }

    #[test]
    fn test_read() {
        let mut i32_cacher = Cacher::<i32>::new();
        let a =  0;
        let v1 = i32_cacher.force_fetch("v1", 10, move || 1 + a);
        assert_eq!(v1, &1);
        let v1 = i32_cacher.read("v1");
        assert_eq!(v1.unwrap(), &1);
        let v1 = i32_cacher.read("v2");
        assert!(v1.is_err());
    }

    #[test]
    fn test_write() {
        let mut i32_cacher = Cacher::<i32>::new();
        let a =  0;
        let v1 = i32_cacher.fetch("v1", 10, move || 1 + a);
        assert_eq!(v1, &1);
        i32_cacher.write("v1", 2).unwrap();
        let v1 = i32_cacher.fetch("v1", 10, move || 1 + a);
        assert_eq!(v1, &2);
    }

    #[test]
    fn test_expire() {
        let mut i32_cacher = Cacher::<i32>::new();
        let a =  0;
        let v1 = i32_cacher.fetch("v1", 10, move || 1 + a);
        assert_eq!(v1, &1);
        i32_cacher.expire("v1").unwrap();
        let v1 = i32_cacher.fetch("v1", 10, move || 2 + a);
        assert_eq!(v1, &2);
    }

    #[test]
    fn test_delete() {
        let mut i32_cacher = Cacher::<i32>::new();
        let a =  0;
        let v1 = i32_cacher.force_fetch("v1", 10, move || 1 + a);
        assert_eq!(v1, &1);
        let v1 = i32_cacher.delete("v1");
        assert!(v1.is_some());
    }
}