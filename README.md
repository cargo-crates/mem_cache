# DEPRECATED
please visite https://github.com/cargo-crates/mem_cache


# CacherRs &emsp;
[![ci](https://github.com/cargo-crates/cacher_rs/workflows/Rust/badge.svg)](https://github.com/cargo-crates/cacher_rs/actions)
[![Latest Version]][crates.io]
![downloads](https://img.shields.io/crates/d/cacher_rs.svg?style=flat-square)

[Latest Version]: https://img.shields.io/crates/v/cacher_rs.svg
[crates.io]: https://crates.io/crates/cacher_rs

### usage

* sync cacher
```rust
use cacher_rs::{Cacher};

let mut i32_cacher = Cacher::<i32>::new();
// expires_in_secs: 0 ->  expires immediate
let v1 = i32_cacher.fetch("v1", 10, || 1);
assert_eq!(v1, &1);

let mut string_cacher = Cacher::<String>::new();
let v1 = string_cacher.fetch("v1", 10, || "1".to_string());
assert_eq!(v1, "1");
```

* async cacher
```rust
use cacher_rs::{AsyncCacher};

let mut i32_cacher = AsyncCacher::<i32>::new();
// expires_in_secs: 0 ->  expires immediate
let v1 = i32_cacher.fetch("v1", 10, || Box::pin(async {
  Ok(1)
})).await?;
assert_eq!(v1, &1);

let mut string_cacher = AsyncCacher::<String>::new();
let v1 = string_cacher.fetch("v1", 10, || Box::pin(async {
  Ok("1".to_string())
})).await?;
assert_eq!(v1, "1");
```

### methods

* `[async] fetch(key, value, closure)` return cache value if not expires or recalculate closure value
* `[async] force_fetch(key, value, closure)` force recalculate closure value
* `[async] read(key)` return key cache value if cache exists
* `write(key, value)` overwrite cache value and expiration time if cache exists
* `expire(key)` make cache value expired if cache exists
* `delete(key)` remove cache if cache exists