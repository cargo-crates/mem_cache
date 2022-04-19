use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::future::Future;
use std::pin::Pin;

pub struct Cache<T> {
  begin_secs: u64,
  expires_in_secs: Duration,
  calculation: Box<dyn Fn() -> Pin<Box<dyn Future<Output = anyhow::Result<T>>>>>,
  value: Option<T>,
}
impl<T> Cache<T> {
  pub fn new(expires_in_secs: u64, calculation: impl Fn() -> Pin<Box<dyn Future<Output = anyhow::Result<T>>>> + 'static) -> Self {
      Cache {
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
  pub async fn value(&mut self) -> anyhow::Result<&T> {
    if self.value.is_none() {
        let value = ((self.calculation)()).await?;
        self.begin_secs = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
        self.value = Some(value);
    }
    Ok(&self.value.as_ref().unwrap())
  }
//   pub fn expires_in_secs(&self) -> u64 {
//       self.expires_in_secs.as_secs()
//   }
}
