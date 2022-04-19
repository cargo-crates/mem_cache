mod cache_item;

use std::marker::PhantomData;
use std::collections::HashMap;

use cache_item::CacheItem;

pub struct Cache<T> {
    cache_item_map: HashMap<String, CacheItem<T>>,
    _mark: PhantomData<T>,
}

impl<T> Cache<T> {
    pub fn new() -> Self {
        Self {
            cache_item_map: HashMap::new(),
            _mark: PhantomData,
        }
    }
    pub fn fetch(&mut self, key: &str, expires_in_secs: u64, calculation: impl Fn() -> T + 'static) -> &T {
        if self.cache_item_map.get(key).is_none() {
            let cache_item: CacheItem<T> = CacheItem::new(expires_in_secs, calculation);
            self.cache_item_map.insert(key.to_string(), cache_item);
            let in_cache_item = self.cache_item_map.get_mut(key).unwrap();
            return in_cache_item.value()
        } else {
            let in_cache_item = self.cache_item_map.get_mut(key).unwrap();
            if in_cache_item.is_value_expires() {
                let cache_item: CacheItem<T> = CacheItem::new(expires_in_secs, calculation);
                *in_cache_item = cache_item
            }
            return in_cache_item.value()
        }
    }
    pub fn force_fetch(&mut self, key: &str, expires_in_secs: u64, calculation: impl Fn() -> T + 'static) -> &T {
        let cache: CacheItem<T> = CacheItem::new(expires_in_secs, calculation);
        self.cache_item_map.insert(key.to_string(), cache);
        self.cache_item_map.get_mut(key).unwrap().value()
    }
    pub fn read(&mut self, key: &str) -> anyhow::Result<&T> {
        match self.cache_item_map.get_mut(key) {
            Some(cache_item) => {
                Ok(cache_item.value())
            },
            None => Err(anyhow::anyhow!("cache not exists"))
        }
    }
    pub fn write(&mut self, key: &str, value: T) -> anyhow::Result<()> {
        match self.cache_item_map.get_mut(key) {
            Some(cache_item) => {
                cache_item.update_value(value);
                Ok(())
            },
            None => Err(anyhow::anyhow!("cache not exists"))
        }
    }
    pub fn expire(&mut self, key: &str) -> anyhow::Result<()> {
        match self.cache_item_map.get_mut(key) {
            Some(cache_item) => {
                cache_item.expire_value();
                Ok(())
            },
            None => Err(anyhow::anyhow!("cache not exists"))
        }
    }
    pub fn delete(&mut self, key: &str) -> Option<CacheItem<T>> {
        self.cache_item_map.remove(key)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    #[test]
    fn test_fetch() {
        let mut i32_cache = Cache::<i32>::new();
        {
            let a =  0;
            let v1 = i32_cache.fetch("v1", 10, move || 1 + a);
            assert_eq!(v1, &1);
        }
        {
            let v2 = i32_cache.fetch("v2", 10, || 2);
            assert_eq!(v2, &2);
        }
        // again
        {
            let v1 = i32_cache.fetch("v1", 10, || 11);
            assert_eq!(v1, &1);
        }
        {
            let v2 = i32_cache.fetch("v2", 10, || 22);
            assert_eq!(v2, &2);
        }
        //  expires
        {
            let v1 = i32_cache.fetch("v1_expires", 3, || 1);
            assert_eq!(v1, &1);
            let v1 = i32_cache.fetch("v1_expires", 0, || 2);
            assert_eq!(v1, &1); // 数据未过期，继续使用旧数据
            let three_secs = time::Duration::from_secs(3);
            thread::sleep(three_secs);
            let v1 = i32_cache.fetch("v1_expires", 0, || 3); // 0 立即失效
            assert_eq!(v1, &3);
            let v1 = i32_cache.fetch("v1_expires", 0, || 4);
            assert_eq!(v1, &4);
        }
        let mut string_cache = Cache::<String>::new();
        {
            let v1 = string_cache.fetch("v1", 10, || "1".to_string());
            assert_eq!(v1, "1");
        }
        {
            let v2 = string_cache.fetch("v2", 10, || "2".to_string());
            assert_eq!(v2, "2");
        }
        // again
        {
            let v1 = string_cache.fetch("v1", 10, || "11".to_string());
            assert_eq!(v1, "1");
        }
        {
            let v2 = string_cache.fetch("v2", 10, || "22".to_string());
            assert_eq!(v2, "2");
        }
    }

    #[test]
    fn test_force_fetch() {
        let mut i32_cache = Cache::<i32>::new();
        let a =  0;
        let v1 = i32_cache.force_fetch("v1", 10, move || 1 + a);
        assert_eq!(v1, &1);
        let v1 = i32_cache.fetch("v1", 10, move || 3 + a);
        assert_eq!(v1, &1);
        let v1 = i32_cache.force_fetch("v1", 10, move || 2 + a);
        assert_eq!(v1, &2);
        let v1 = i32_cache.fetch("v1", 10, move || 3 + a);
        assert_eq!(v1, &2);
    }

    #[test]
    fn test_read() {
        let mut i32_cache = Cache::<i32>::new();
        let a =  0;
        let v1 = i32_cache.force_fetch("v1", 10, move || 1 + a);
        assert_eq!(v1, &1);
        let v1 = i32_cache.read("v1");
        assert_eq!(v1.unwrap(), &1);
        let v1 = i32_cache.read("v2");
        assert!(v1.is_err());
    }

    #[test]
    fn test_write() {
        let mut i32_cache = Cache::<i32>::new();
        let a =  0;
        let v1 = i32_cache.fetch("v1", 10, move || 1 + a);
        assert_eq!(v1, &1);
        i32_cache.write("v1", 2).unwrap();
        let v1 = i32_cache.fetch("v1", 10, move || 1 + a);
        assert_eq!(v1, &2);
    }

    #[test]
    fn test_expire() {
        let mut i32_cache = Cache::<i32>::new();
        let a =  0;
        let v1 = i32_cache.fetch("v1", 10, move || 1 + a);
        assert_eq!(v1, &1);
        i32_cache.expire("v1").unwrap();
        let v1 = i32_cache.fetch("v1", 10, move || 2 + a);
        assert_eq!(v1, &2);
    }

    #[test]
    fn test_delete() {
        let mut i32_cache = Cache::<i32>::new();
        let a =  0;
        let v1 = i32_cache.force_fetch("v1", 10, move || 1 + a);
        assert_eq!(v1, &1);
        let v1 = i32_cache.delete("v1");
        assert!(v1.is_some());
    }
}