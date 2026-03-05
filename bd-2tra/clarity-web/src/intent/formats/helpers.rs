#[must_use]
pub const fn is_leap_year(year: i32) -> bool {
  (year % 4 == 0) && ((year % 100 != 0) || (year % 400 == 0))
}

#[must_use]
pub const fn get_days_in_month(month: u8, is_leap: bool) -> u8 {
  match month {
    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
    4 | 6 | 9 | 11 => 30,
    2 if is_leap => 29,
    2 => 28,
    _ => 0,
  }
}

#[must_use]
pub const fn is_valid_hex_char(c: char) -> bool {
  c.is_ascii_hexdigit()
}

#[must_use]
pub fn is_valid_hex(s: &str) -> bool {
  !s.is_empty() && s.chars().all(is_valid_hex_char)
}
