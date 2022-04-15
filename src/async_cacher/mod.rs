mod async_cache;

use std::marker::PhantomData;
use std::collections::HashMap;
use std::future::Future;

use async_cache::Cache;
use std::pin::Pin;

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
    pub async fn fetch(&mut self, key: &str, expires_in_secs: u64, calculation: impl Fn() -> Pin<Box<dyn Future<Output = anyhow::Result<T>>>> + 'static) -> anyhow::Result<&T> {
        let cache: Cache<T> = Cache::new(expires_in_secs, calculation);
        if self.cache_map.get(key).is_none() {
            self.cache_map.insert(key.to_string(), cache);
            let in_cache = self.cache_map.get_mut(key).unwrap();
            return in_cache.value().await
        } else {
            let in_cache = self.cache_map.get_mut(key).unwrap();
            if in_cache.is_value_expires() {
                *in_cache = cache
            }
            return in_cache.value().await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    async fn main_test() -> anyhow::Result<()> {
        let mut i32_cacher = Cacher::<i32>::new();
        {
            let v1 = i32_cacher.fetch("v1", 10, || Box::pin(async { Ok(1)})).await?;
            assert_eq!(v1, &1);
        }
         //  expires
         {
            let v1 = i32_cacher.fetch("v1_expires", 3, || Box::pin(async { Ok(1)})).await?;
            assert_eq!(v1, &1);
            let v1 = i32_cacher.fetch("v1_expires", 0, || Box::pin(async { Ok(2)})).await?;
            assert_eq!(v1, &1); // 数据未过期，继续使用旧数据
            let three_secs = time::Duration::from_secs(3);
            thread::sleep(three_secs);
            let v1 = i32_cacher.fetch("v1_expires", 0, || Box::pin(async { Ok(3)})).await?; // 0 立即失效
            assert_eq!(v1, &3);
            let v1 = i32_cacher.fetch("v1_expires", 0, || Box::pin(async { Ok(4)})).await?;
            assert_eq!(v1, &4);
        }
        let mut string_cacher = Cacher::<String>::new();
        {
            let v1 = string_cacher.fetch("v1", 10, || Box::pin(async { Ok("1".to_string())})).await?;
            assert_eq!(v1, "1");
        }

        Ok(())
    }

    #[test]
    fn test_cacher() {
        assert!(
            match tokio_test::block_on(main_test()) {
                Ok(()) => Ok(()),
                Err(e) => {
                    eprintln!("err: {:?}", e);
                    Err(e)
                }
            }.is_ok());
    }

}