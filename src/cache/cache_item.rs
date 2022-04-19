use std::time::{SystemTime, UNIX_EPOCH, Duration};

pub struct CacheItem<T> {
  begin_secs: u64,
  expires_in_secs: Duration,
  calculation: Box<dyn Fn() -> T>,
  value: Option<T>,
}
impl<T> CacheItem<T> {
  pub fn new(expires_in_secs: u64, calculation: impl Fn() -> T + 'static) -> Self {
    CacheItem {
          begin_secs: 0,
          expires_in_secs: Duration::new(expires_in_secs, 0),
          calculation: Box::new(calculation),
          value: None
      }
  }
  pub fn is_value_expires(&mut self) -> bool {
      let current_secs = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
      if current_secs >= self.begin_secs + self.expires_in_secs.as_secs() {
          true
      } else {
          false
      }
  }
  pub fn update_value(&mut self, value: T) {
    self.begin_secs = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
    self.value = Some(value);
  }
  pub fn expire_value(&mut self) {
    self.begin_secs = 0;
    self.value = None;
  }
  pub fn value(&mut self) -> &T {
      self.value.get_or_insert_with(|| {
          let v = (self.calculation)();
          self.begin_secs = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
          v
      })
  }
//   pub fn expires_in_secs(&self) -> u64 {
//       self.expires_in_secs.as_secs()
//   }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cache_item() {
      let mut i32_cache_item = CacheItem::new(10, || 5);
      assert_eq!(i32_cache_item.value(), &5);
  }
}