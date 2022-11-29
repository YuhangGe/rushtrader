use chrono::{DateTime, Offset, Utc};
use chrono_tz::America::New_York;

#[inline]
pub fn get_market_close_utc_hours(dt: &DateTime<Utc>) -> u32 {
  if dt.with_timezone(&New_York).offset().fix().local_minus_utc() == -14400 {
    // EDT
    21
  } else {
    // EST
    22
  }
}

#[macro_export]
macro_rules! bool_map {
  ($exp: expr , $true_exp: expr , $false_exp: expr) => {
    if $exp {
      $true_exp
    } else {
      $false_exp
    }
  };
}
