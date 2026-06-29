use std::{fmt::Display, num::ParseIntError, ops::Add, str::FromStr};

use serde::{Deserialize, Serialize};

/// A base-36 encoded unsigned integer, using digits `0-9` and `a-z`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Clone, Copy, Hash)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct Base36(u32);

impl Base36 {
    /// Creates a `Base36` from a raw `u32`. Only available in tests.
    #[cfg(test)]
    pub fn new(n: u32) -> Self {
        Self(n)
    }
}

impl Add for Base36 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<u32> for Base36 {
    type Output = Self;
    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl FromStr for Base36 {
    type Err = ParseIntError;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let num = u32::from_str_radix(s, 36)?;
        Ok(Self(num))
    }
}

impl Display for Base36 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut num = self.0;

        let mut digits = ['\0'; 7];
        let mut idx = 0;

        loop {
            if let Some(digit) = char::from_digit(num % 36, 36)
                && let Some(ch) = digits.get_mut(idx)
            {
                *ch = digit;
                idx += 1;
                num /= 36;
            }

            if num == 0 {
                break;
            }
        }

        for ch in digits.iter().take(idx).rev() {
            write!(f, "{ch}")?;
        }

        Ok(())
    }
}

impl TryFrom<String> for Base36 {
    type Error = ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<Base36> for String {
    fn from(val: Base36) -> Self {
        val.to_string()
    }
}

impl PartialEq<u32> for Base36 {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn base36_display_zero() {
        assert_eq!("0", Base36(0).to_string());
    }

    #[test]
    fn base36_display_single_digit() {
        assert_eq!("9", Base36(9).to_string());
    }

    #[test]
    fn base36_display_first_alpha_digit() {
        assert_eq!("a", Base36(10).to_string());
    }

    #[test]
    fn base36_display_last_single_digit() {
        assert_eq!("z", Base36(35).to_string());
    }

    #[test]
    fn base36_display_multi_digit() {
        assert_eq!("idg", Base36(23812).to_string());
    }

    #[test]
    fn base36_display_u32_max() {
        assert_eq!("1z141z3", Base36(u32::MAX).to_string());
    }

    #[test]
    fn base36_parse_zero() {
        assert_eq!(Base36(0), "0".parse::<Base36>().unwrap());
    }

    #[test]
    fn base36_parse_single_digit() {
        assert_eq!(Base36(9), "9".parse::<Base36>().unwrap());
    }

    #[test]
    fn base36_parse_first_alpha_digit() {
        assert_eq!(Base36(10), "a".parse::<Base36>().unwrap());
    }

    #[test]
    fn base36_parse_last_single_digit() {
        assert_eq!(Base36(35), "z".parse::<Base36>().unwrap());
    }

    #[test]
    fn base36_parse_multi_digit() {
        assert_eq!(Base36(23812), "idg".parse::<Base36>().unwrap());
    }

    #[test]
    fn base36_parse_post_id_example() {
        assert_eq!(Base36(49), "1d".parse::<Base36>().unwrap());
    }

    #[test]
    fn base36_parse_errors_on_invalid_input() {
        assert!("!@#".parse::<Base36>().is_err());
        assert!("".parse::<Base36>().is_err());
    }

    #[test]
    fn base36_roundtrip() {
        for n in [0, 9, 10, 35, 23812, u32::MAX] {
            let encoded = Base36(n).to_string();
            let decoded: Base36 = encoded.parse().unwrap();
            assert_eq!(Base36(n), decoded, "roundtrip failed for {n}");
        }
    }

    #[test]
    fn base36_add_base36() {
        assert_eq!(Base36(3), Base36(1) + Base36(2));
    }

    #[test]
    fn base36_add_u32() {
        assert_eq!(Base36(3), Base36(1) + 2u32);
    }
}
