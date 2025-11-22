#![allow(clippy::expect_used)]

use std::{fmt::Display, num::ParseIntError, ops::Add, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Clone, Copy, Hash)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct Base36(u32);

impl Base36 {
    #[cfg(test)]
    pub fn new(n: u32) -> Self {
        Self(n)
    }
}

impl Add for Base36 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut this = self;
        this.0 += rhs.0;
        this
    }
}

impl Add<u32> for Base36 {
    type Output = Self;
    fn add(self, rhs: u32) -> Self::Output {
        let mut this = self;
        this.0 += rhs;
        this
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
        let mut buffer = ['\0'; 128];

        let mut len = 0;

        loop {
            assert!(
                len < buffer.len(),
                "a proper `u32` to a base36 `char` conversion would never loop more than the stack allocated array len"
            );

            let codepoint = num % 36;
            num /= 36;

            *buffer
                .get_mut(len)
                .expect("index must be valid due to assertion above") =
                std::char::from_digit(codepoint, 36).expect("base36 digit must always be valid");

            len += 1;

            if num == 0 {
                break;
            }
        }

        for c in buffer.iter().take(len).rev() {
            write!(f, "{c}")?;
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
mod test {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn should_format_u32_to_base_36() {
        {
            let num = Base36(0);
            pretty_assertions::assert_str_eq!("0", num.to_string());
        }
        {
            let num = Base36(9);
            pretty_assertions::assert_str_eq!("9", num.to_string());
        }
        {
            let num = Base36(10);
            pretty_assertions::assert_str_eq!("a", num.to_string());
        }
        {
            let num = Base36(35);
            pretty_assertions::assert_str_eq!("z", num.to_string());
        }
        {
            let num = Base36(23812);
            pretty_assertions::assert_str_eq!("idg", num.to_string());
        }
        {
            let num = Base36(u32::MAX);
            pretty_assertions::assert_str_eq!("1z141z3", num.to_string());
        }
    }

    #[test]
    fn should_parse_base_36_from_str() {
        {
            let num: Base36 = "0".parse().unwrap();

            pretty_assertions::assert_eq!(num, 0);
        }
        {
            let num: Base36 = "9".parse().unwrap();

            pretty_assertions::assert_eq!(num, 9);
        }
        {
            let num: Base36 = "a".parse().unwrap();

            pretty_assertions::assert_eq!(num, 10);
        }
        {
            let num: Base36 = "z".parse().unwrap();

            pretty_assertions::assert_eq!(num, 35);
        }
        {
            let num: Base36 = "idg".parse().unwrap();

            pretty_assertions::assert_eq!(num, 23812);
        }
        {
            let num: Base36 = "1d".parse().unwrap();

            pretty_assertions::assert_eq!(num, 49);
        }
    }
}
