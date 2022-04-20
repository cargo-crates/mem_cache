# MemCache &emsp;
[![ci](https://github.com/cargo-crates/mem_cache/workflows/Rust/badge.svg)](https://github.com/cargo-crates/mem_cache/actions)
[![Latest Version]][crates.io]
![downloads](https://img.shields.io/crates/d/mem_cache.svg?style=flat-square)

[Latest Version]: https://img.shields.io/crates/v/mem_cache.svg
[crates.io]: https://crates.io/crates/mem_cache

### usage

* cache
```rust
use mem_cache::{Cache};

let mut i32_cache = Cache::<i32>::new();
// expires_in_secs: 0 -> Immediately expired
let v1 = i32_cache.fetch("v1", 0, || 1);
assert_eq!(v1, &1);

let mut string_cache = Cache::<String>::new();
let v1 = string_cache.fetch("v1", 10, || "1".to_string());
assert_eq!(v1, "1");
```

* async cache
```rust
use mem_cache::{AsyncCache};

let mut i32_cache = AsyncCache::<i32>::new();
// expires_in_secs: 0 -> Immediately expired
let v1 = i32_cache.fetch("v1", 0, || Box::pin(async {
  Ok(1)
})).await?;
assert_eq!(v1, &1);

let mut string_cache = AsyncCache::<String>::new();
let v1 = string_cache.fetch("v1", 10, || Box::pin(async {
  Ok("1".to_string())
})).await?;
assert_eq!(v1, "1");
```

### methods

* `[async] fetch(key, expires_in_secs, closure)` return cache value if not expires or recalculate closure value
* `[async] force_fetch(key, expires_in_secs, closure)` force recalculate closure value
* `[async] get(key)` return key cache value if cache exists
* `keys()` return all cached keys including the expired cache
* `insert(key, value)` overwrite cache value and expiration time if cache exists
* `expire(key)` make cache value expired if cache exists
* `contains_key(key)` returns true if the cache contains an entry for the given key
* `remove(key)` remove cache if cache exists
* `clear_expired()` cleanups the cache by removing expired entries.
* `clear()` empty all data include valid cache.