use std::fmt::Display;

#[derive(Debug)]
pub struct Position {
  pub size: isize,
  pub price: f64,
  pub origin_price: f64,
}

impl Position {
  pub fn new() -> Self {
    Self {
      size: 0,
      price: 0.,
      origin_price: 0.,
    }
  }
}
impl Default for Position {
  fn default() -> Self {
    Self::new()
  }
}
impl Display for Position {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "size: {}, price: {}, origin_price: {}",
      self.size, self.price, self.origin_price
    )
  }
}
