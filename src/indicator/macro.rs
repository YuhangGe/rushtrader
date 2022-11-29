#[macro_export]
macro_rules! impl_indicator_trait {
  ($indicator: ident) => {
    impl DataLineFeed for $indicator {
      #[inline(always)]
      fn inner(&self) -> (&[f64], usize) {
        (&self.data, self.start_pos)
      }
    }
    impl DataLine for $indicator {
      #[inline(always)]
      fn at(&self, index: usize) -> Option<f64> {
        get_vec_at(&self.data, self.start_pos, index)
      }
    }
  };
}
#[macro_export]
macro_rules! impl_indicator_without_period {
  ($indicator: ident) => {
    impl $indicator {
      pub fn new() -> Self {
        Self {
          data: Vec::new(),
          start_pos: 0,
        }
      }
    }
    impl Default for $indicator {
      fn default() -> Self {
        Self::new()
      }
    }
  };
}
#[macro_export]
macro_rules! impl_indicator_with_period {
  ($indicator: ident) => {
    impl_indicator_with_period!($indicator, 2);
  };
  ($indicator: ident, $min_period: literal) => {
    impl $indicator {
      pub fn new(period: usize) -> Self {
        if period < $min_period {
          panic!("sma period must gte {}", $min_period);
        }
        Self {
          data: Vec::new(),
          period,
          start_pos: 0,
        }
      }
    }
  };
}
