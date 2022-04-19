mod cache_item;

use std::marker::PhantomData;
use std::collections::HashMap;
use std::future::Future;

use cache_item::CacheItem;
use std::pin::Pin;

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
    pub async fn fetch(&mut self, key: &str, expires_in_secs: u64, calculation: impl Fn() -> Pin<Box<dyn Future<Output = anyhow::Result<T>>>> + 'static) -> anyhow::Result<&T> {
        let cache_item: CacheItem<T> = CacheItem::new(expires_in_secs, calculation);
        if self.cache_item_map.get(key).is_none() {
            self.cache_item_map.insert(key.to_string(), cache_item);
            let in_cache_item = self.cache_item_map.get_mut(key).unwrap();
            return in_cache_item.value().await
        } else {
            let in_cache_item = self.cache_item_map.get_mut(key).unwrap();
            if in_cache_item.is_value_expires() {
                *in_cache_item = cache_item
            }
            return in_cache_item.value().await
        }
    }
    pub async fn force_fetch(&mut self, key: &str, expires_in_secs: u64, calculation: impl Fn() -> Pin<Box<dyn Future<Output = anyhow::Result<T>>>> + 'static) -> anyhow::Result<&T> {
        let cache_item: CacheItem<T> = CacheItem::new(expires_in_secs, calculation);
        self.cache_item_map.insert(key.to_string(), cache_item);
        self.cache_item_map.get_mut(key).unwrap().value().await
    }
    pub async fn read(&mut self, key: &str) -> anyhow::Result<&T> {
        match self.cache_item_map.get_mut(key) {
            Some(cache_item) => {
                cache_item.value().await
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

    async fn test_fetch() -> anyhow::Result<()> {
        let mut i32_cache = Cache::<i32>::new();
        {
            let v1 = i32_cache.fetch("v1", 10, || Box::pin(async { Ok(1)})).await?;
            assert_eq!(v1, &1);
        }
         //  expires
         {
            let v1 = i32_cache.fetch("v1_expires", 3, || Box::pin(async { Ok(1)})).await?;
            assert_eq!(v1, &1);
            let v1 = i32_cache.fetch("v1_expires", 0, || Box::pin(async { Ok(2)})).await?;
            assert_eq!(v1, &1); // 数据未过期，继续使用旧数据
            let three_secs = time::Duration::from_secs(3);
            thread::sleep(three_secs);
            let v1 = i32_cache.fetch("v1_expires", 0, || Box::pin(async { Ok(3)})).await?; // 0 立即失效
            assert_eq!(v1, &3);
            let v1 = i32_cache.fetch("v1_expires", 0, || Box::pin(async { Ok(4)})).await?;
            assert_eq!(v1, &4);
        }
        let mut string_cache = Cache::<String>::new();
        {
            let v1 = string_cache.fetch("v1", 10, || Box::pin(async { Ok("1".to_string())})).await?;
            assert_eq!(v1, "1");
        }

        Ok(())
    }

    async fn test_force_fetch() -> anyhow::Result<()> {
        let mut i32_cache = Cache::<i32>::new();
        let v1 = i32_cache.force_fetch("v1", 10, || Box::pin(async { Ok(1)})).await?;
        assert_eq!(v1, &1);
        let v1 = i32_cache.fetch("v1", 10, || Box::pin(async { Ok(3)})).await?;
        assert_eq!(v1, &1);
        let v1 = i32_cache.force_fetch("v1", 10, || Box::pin(async { Ok(2)})).await?;
        assert_eq!(v1, &2);
        let v1 = i32_cache.fetch("v1", 10, || Box::pin(async { Ok(3)})).await?;
        assert_eq!(v1, &2);

        Ok(())
    }

    async fn test_read() -> anyhow::Result<()> {
        let mut i32_cache = Cache::<i32>::new();
        let v1 = i32_cache.force_fetch("v1", 10, || Box::pin(async { Ok(1)})).await?;
        assert_eq!(v1, &1);
        let v1 = i32_cache.read("v1").await;
        assert_eq!(v1.unwrap(), &1);
        let v1 = i32_cache.read("v2").await;
        assert!(v1.is_err());

        Ok(())
    }

    async fn test_write() -> anyhow::Result<()> {
        let mut i32_cache = Cache::<i32>::new();
        let v1 = i32_cache.fetch("v1", 10, || Box::pin(async { Ok(1)})).await?;
        assert_eq!(v1, &1);
        i32_cache.write("v1", 3).unwrap();
        let v1 = i32_cache.fetch("v1", 10, || Box::pin(async { Ok(1)})).await?;
        assert_eq!(v1, &3);

        Ok(())
    }

    async fn test_expire() -> anyhow::Result<()> {
        let mut i32_cache = Cache::<i32>::new();
        let v1 = i32_cache.fetch("v1", 10, || Box::pin(async { Ok(1)})).await?;
        assert_eq!(v1, &1);
        i32_cache.expire("v1").unwrap();
        let v1 = i32_cache.fetch("v1", 10, || Box::pin(async { Ok(2)})).await?;
        assert_eq!(v1, &2);

        Ok(())
    }

    async fn test_delete() -> anyhow::Result<()> {
        let mut i32_cache = Cache::<i32>::new();
        let v1 = i32_cache.force_fetch("v1", 10, || Box::pin(async { Ok(1)})).await?;
        assert_eq!(v1, &1);
        let v1 = i32_cache.delete("v1");
        assert!(v1.is_some());

        Ok(())
    }

    #[test]
    fn test_cache() {

        async fn test_all() -> anyhow::Result<()> {
            test_fetch().await?;
            test_force_fetch().await?;
            test_read().await?;
            test_write().await?;
            test_expire().await?;
            test_delete().await?;

            Ok(())
        }


        assert!(
            match tokio_test::block_on(test_all()) {
                Ok(()) => Ok(()),
                Err(e) => {
                    eprintln!("err: {:?}", e);
                    Err(e)
                }
            }.is_ok());
    }

}