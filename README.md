### usage

* sync cache
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

* async cache
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