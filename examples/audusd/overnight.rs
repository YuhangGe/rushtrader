use std::fmt::Display;

#[derive(Debug)]
pub(super) enum OvernightType {
  Long,
  Short,
  None,
}
impl Display for OvernightType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let v = match self {
      OvernightType::Long => "long",
      OvernightType::Short => "short",
      _ => "none",
    };
    write!(f, "{}", v)
  }
}

#[derive(Debug)]
pub(super) enum OvernightSignal {
  Long(f64),
  Short(f64),
  None,
}
impl Display for OvernightSignal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let v = match self {
      OvernightSignal::Long(_) => "long",
      OvernightSignal::Short(_) => "short",
      _ => "none",
    };
    write!(f, "{}", v)
  }
}
pub(super) struct Overnight {
  pub position: isize,
  pub overtype: OvernightType,
  pub signal: OvernightSignal,
}

impl Overnight {
  pub fn new() -> Self {
    Self {
      position: 0,
      overtype: OvernightType::None,
      signal: OvernightSignal::None,
    }
  }
}
