use std::{num::ParseIntError, str::FromStr};

use anyhow::Context;
use fraction::{GenericFraction, Sign, generic::GenericInteger};
use num::pow::Pow;
use num::{Zero, integer::Integer};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MyFrac<T>
where
    T: Integer + Clone + Copy,
{
    pub sign: bool,
    pub num: T,
    pub den: T,
}

impl<T> From<GenericFraction<T>> for MyFrac<T>
where
    T: Integer + Clone + Copy,
{
    fn from(value: GenericFraction<T>) -> Self {
        match value {
            GenericFraction::Rational(sign, ratio) => {
                let (num, den) = ratio.into_raw();
                MyFrac {
                    sign: sign == Sign::Plus,
                    num,
                    den,
                }
            }
            GenericFraction::Infinity(sign) => MyFrac {
                sign: sign == Sign::Plus,
                num: T::one(),
                den: T::zero(),
            },
            GenericFraction::NaN => MyFrac {
                sign: true,
                num: T::zero(),
                den: T::zero(),
            },
        }
    }
}

impl<T> Into<GenericFraction<T>> for MyFrac<T>
where
    T: Integer + Clone + Copy,
{
    fn into(self) -> GenericFraction<T> {
        if self.den == T::zero() {
            if self.num == T::zero() {
                GenericFraction::NaN
            } else {
                let sign = if self.sign { Sign::Plus } else { Sign::Minus };
                GenericFraction::Infinity(sign)
            }
        } else {
            let sign = if self.sign { Sign::Plus } else { Sign::Minus };
            GenericFraction::new_raw_signed(sign, self.num, self.den)
        }
    }
}

fn limit_den_helper<T>((num, den): (T, T), max_den: T) -> Result<(T, T), String>
where
    T: Integer + Clone + Copy,
{
    if max_den > T::zero() {
        let (mut p0, mut q0, mut p1, mut q1) = (T::zero(), T::one(), T::one(), T::zero());
        let (mut n, mut d) = (num, den);
        loop {
            let a = n / d;
            let q2 = q0 + a * q1;
            if q2 > max_den {
                break;
            }
            let (new_p0, new_q0) = (p1, q1);
            (p1, q1) = (p0 + a * p1, q2);
            (p0, q0) = (new_p0, new_q0);
            (n, d) = (d, n - a * d)
        }
        let k = (max_den - q0) / q1;
        if (d + d) * (q0 + k * q1) <= den {
            Ok((p1, q1))
        } else {
            Ok((p0 + k * p1, q0 + k * q1))
        }
    } else {
        Err("max_den must be positive".to_string())
    }
}

impl<T> MyFrac<T>
where
    T: Integer + Clone + Copy,
{
    pub fn limit_den(self: MyFrac<T>, max_den: T) -> MyFrac<T> {
        let MyFrac { sign, num, den } = self;
        if den == T::zero() || den <= max_den {
            self
        } else {
            let (num, den) = limit_den_helper((num, den), max_den).unwrap();
            MyFrac { sign, num, den }
        }
    }
}

impl Pow<i64> for MyFrac<u64> {
    type Output = Self;

    fn pow(self, rhs: i64) -> Self::Output {
        if rhs == 0 {
            return MyFrac {
                sign: true,
                num: 1,
                den: 1,
            };
        }
        if rhs == 1 {
            return self;
        }
        let MyFrac { sign, num, den } = self;
        let sign = sign || rhs % 2 == 0;
        if rhs > 0 {
            let (num, den) = (num.pow(rhs as u32), den.pow(rhs as u32));
            MyFrac { sign, num, den }
        } else {
            let (num, den) = (den.pow((-rhs) as u32), num.pow((-rhs) as u32));
            MyFrac { sign, num, den }
        }
    }
}

fn split_decimal_notation(src: &str) -> Result<(Sign, String, String, isize), anyhow::Error> {
    let (src, mut exp) = if let Some(idx) = src.find(&['E', 'e']) {
        (
            &src[..idx],
            src[idx + 1..]
                .parse::<isize>()
                .context("Invalid exponent value.")?,
        )
    } else {
        (&src[..], 0)
    };
    let (src, sign) = match src.chars().nth(0) {
        Some('-') => (&src[1..], Sign::Minus),
        Some('+') => (&src[1..], Sign::Plus),
        _ => (&src[..], Sign::Plus),
    };
    if let Some(idx) = src.find('.') {
        // has decimal point
        let before_point = &src[..idx];
        let after_point = &src[idx + 1..];

        match (before_point.find('['), before_point.find(']')) {
            (Some(l_idx), Some(r_idx)) => {
                // 1[23]4.5678
                let int_part = &before_point[..l_idx];
                let repeating_part = &before_point[l_idx + 1..r_idx];
                exp += (before_point[r_idx + 1..].len() + repeating_part.len()) as isize;
                Ok((sign, int_part.to_string(), repeating_part.to_string(), exp))
            }
            (Some(l_idx), None) => {
                // 12[34.5]678
                let r_idx = after_point
                    .rfind(']')
                    .context("Bracket for repeating part not closed")?;
                let int_part = before_point.to_string();
                let before_point_repeating_digits = &before_point[l_idx + 1..];
                exp += before_point_repeating_digits.len() as isize;
                let mut repeating_part = before_point_repeating_digits.to_string();
                repeating_part.push_str(&after_point[..r_idx]);
                Ok((sign, int_part, repeating_part, exp))
            }
            (None, None) => {
                match (after_point.find('['), after_point.rfind(']')) {
                    // 1234.5[67]8
                    (Some(l_idx), Some(r_idx)) => {
                        let mut int_part = before_point.to_string();
                        let after_point_int_part = &after_point[..l_idx];
                        exp -= after_point_int_part.len() as isize;
                        int_part.push_str(after_point_int_part);
                        let repeating_part = &after_point[l_idx + 1..r_idx];
                        Ok((sign, int_part, repeating_part.to_string(), exp))
                    }
                    (None, None) => {
                        // 1234.5678
                        let mut int_part = before_point.to_string();
                        int_part.push_str(after_point);
                        exp -= after_point.len() as isize;
                        Ok((sign, int_part, "".to_string(), exp))
                    }
                    _ => anyhow::bail!("Bracket for repeating part not match"),
                }
            }
            _ => anyhow::bail!("Starting bracket for repeating part not found"),
        }
    } else {
        // no decimal point
        match (src.find('['), src.rfind(']')) {
            (Some(l_idx), Some(r_idx)) => {
                // 123[456]78
                let int_part = &src[..l_idx];
                let repeating_part = &src[l_idx + 1..r_idx];
                exp += repeating_part.len() as isize;
                Ok((sign, int_part.to_string(), repeating_part.to_string(), exp))
            }
            (None, None) => {
                // 12345678
                Ok((sign, src.to_string(), "".to_string(), exp))
            }
            _ => anyhow::bail!("Invalid fraction format"),
        }
    }
}

fn empty_safe_parse<T>(src: &str) -> Result<T, ParseIntError>
where
    T: FromStr<Err = ParseIntError> + Zero,
{
    if src.is_empty() {
        Ok(T::zero())
    } else {
        T::from_str(src)
    }
}

fn fraction_from_decimal<T>(
    sign: Sign,
    int_part: &str,
    repeating_part: &str,
    exp: isize,
) -> Result<GenericFraction<T>, anyhow::Error>
where
    T: GenericInteger
        + Clone
        + Copy
        + FromStr<Err = ParseIntError>
        + Pow<usize, Output = T>
        + Into<GenericFraction<T>>,
{
    if !int_part.chars().all(|c| c.is_digit(10)) {
        anyhow::bail!("Invalid integer part")
    }
    if !repeating_part.chars().all(|c| c.is_digit(10)) {
        anyhow::bail!("Invalid repeating part")
    }
    let repeating_part_len = repeating_part.len();
    let int_part: T = empty_safe_parse(int_part)?;
    let mut result: GenericFraction<T> = int_part.into();
    if repeating_part_len > 0 {
        let repeat_den: T = T::_10().pow(repeating_part_len) - T::_1();
        let repeat_num: T = empty_safe_parse(repeating_part)?;
        result += if let Some(fr) = GenericFraction::new_generic(Sign::Plus, repeat_num, repeat_den)
        {
            fr
        } else {
            anyhow::bail!("Invalid fraction format")
        };
    }
    if exp > 0 {
        result *= T::_10().pow(exp as usize);
    } else if exp < 0 {
        result /= T::_10().pow((-exp) as usize);
    }
    match sign {
        Sign::Plus => Ok(result),
        Sign::Minus => Ok(-result),
    }
}

pub fn parse_fraction<T>(src: &str) -> Result<GenericFraction<T>, anyhow::Error>
where
    T: GenericInteger
        + Clone
        + Copy
        + FromStr<Err = ParseIntError>
        + Pow<usize, Output = T>
        + Into<GenericFraction<T>>,
{
    if src.eq_ignore_ascii_case("inf") {
        return Ok(GenericFraction::infinity());
    } else if src.eq_ignore_ascii_case("-inf") {
        return Ok(GenericFraction::neg_infinity());
    } else if src.eq_ignore_ascii_case("nan") {
        return Ok(GenericFraction::nan());
    }
    match src.find('/') {
        Some(_) => {
            // let num = T::from_str(&src[..idx])?;
            // let den = T::from_str(&src[idx + 1..])?;
            // if let Some(fr) = GenericFraction::new_generic(Sign::Plus, num, den) {
            //     Ok(fr)
            // } else {
            //     anyhow::bail!("Invalid fraction format")
            // }
            // [TODO]
            Ok(GenericFraction::from_str(src)?)
        }
        None => {
            let (sign, int_part, repeating_part, exp) = split_decimal_notation(src)?;
            fraction_from_decimal(sign, &int_part, &repeating_part, exp)
        }
    }
}
