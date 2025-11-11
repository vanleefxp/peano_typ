use std::{ops::*, str::FromStr};

use crate::traits::*;
use anyhow::{Context, anyhow, bail};
use malachite::base::num::{arithmetic::traits::*, basic::traits::*};

pub enum ParseFractionResult<T> {
    Rational(bool, T, T),
    Inf(bool),
    Zero(bool),
    NaN,
}

struct FractionFromDecimalResult {
    sign: bool,
    int_part: String,
    repeating_part: String,
    exp: isize,
}

fn split_decimal_notation(src: &str) -> Result<FractionFromDecimalResult, anyhow::Error> {
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
        Some('-') => (&src[1..], false),
        Some('+') => (&src[1..], true),
        _ => (&src[..], true),
    };
    if let Some(idx) = src.find('.') {
        // has decimal point
        let before_point = &src[..idx];
        let after_point = &src[idx + 1..];

        match (before_point.find('['), before_point.rfind(']')) {
            (Some(l_idx), Some(r_idx)) => {
                // 1[23]4.5678
                let int_part = &before_point[..l_idx];
                let repeating_part = &before_point[l_idx + 1..r_idx];
                exp += (before_point[r_idx + 1..].len() + repeating_part.len()) as isize;
                let int_part = int_part.to_string();
                let repeating_part = repeating_part.to_string();
                Ok(FractionFromDecimalResult {
                    sign,
                    int_part,
                    repeating_part,
                    exp,
                })
            }
            (Some(l_idx), None) => {
                // 12[34.5]678
                let r_idx = after_point
                    .rfind(']')
                    .context("Bracket for repeating part not closed")?;
                let int_part = before_point[..l_idx].to_string();
                let before_point_repeating_digits = &before_point[l_idx + 1..];
                exp += before_point_repeating_digits.len() as isize;
                let mut repeating_part = before_point_repeating_digits.to_string();
                repeating_part.push_str(&after_point[..r_idx]);
                Ok(FractionFromDecimalResult {
                    sign,
                    int_part,
                    repeating_part,
                    exp,
                })
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
                        let repeating_part = repeating_part.to_string();
                        Ok(FractionFromDecimalResult {
                            sign,
                            int_part,
                            repeating_part,
                            exp,
                        })
                    }
                    (None, None) => {
                        // 1234.5678
                        let mut int_part = before_point.to_string();
                        int_part.push_str(after_point);
                        exp -= after_point.len() as isize;
                        Ok(FractionFromDecimalResult {
                            sign,
                            int_part,
                            repeating_part: "".to_string(),
                            exp,
                        })
                    }
                    _ => bail!("Bracket for repeating part not match"),
                }
            }
            _ => bail!("Starting bracket for repeating part not found"),
        }
    } else {
        // no decimal point
        match (src.find('['), src.rfind(']')) {
            (Some(l_idx), Some(r_idx)) => {
                // 123[456]78
                let int_part = &src[..l_idx];
                let repeating_part = &src[l_idx + 1..r_idx];
                exp += repeating_part.len() as isize;
                let int_part = int_part.to_string();
                let repeating_part = repeating_part.to_string();
                Ok(FractionFromDecimalResult {
                    sign,
                    int_part,
                    repeating_part,
                    exp,
                })
            }
            (None, None) => {
                // 12345678
                Ok(FractionFromDecimalResult {
                    sign,
                    int_part: src.to_string(),
                    repeating_part: "".to_string(),
                    exp,
                })
            }
            _ => bail!("Invalid fraction format"),
        }
    }
}

fn fraction_from_decimal<T, E>(
    from_decimal_result: FractionFromDecimalResult,
) -> Result<ParseFractionResult<T>, anyhow::Error>
where
    T: Clone
        + FromStr<Err = E>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + for<'a> MulAssign<&'a T>
        + AddAssign
        + PartialEq
        + Pow<u64, Output = T>
        + Zero
        + One
        + Ten,
{
    let FractionFromDecimalResult {
        sign,
        int_part,
        repeating_part,
        exp,
    } = from_decimal_result;
    let int_part = &int_part[..];
    let repeating_part = &repeating_part[..];
    if !int_part.chars().all(|c| c.is_digit(10)) {
        bail!("Invalid integer part")
    }
    if !repeating_part.chars().all(|c| c.is_digit(10)) {
        bail!("Invalid repeating part")
    }
    let repeating_part_len = repeating_part.len() as u64;

    let mut num: T = if int_part.is_empty() {
        T::ZERO
    } else {
        T::from_str(int_part).map_err(|_| anyhow!("parsing failed"))?
    };
    let mut den: T = if repeating_part_len > 0 {
        let repeat_den: T = T::TEN.pow(repeating_part_len) - T::ONE;
        let repeat_num: T = if repeating_part.is_empty() {
            T::ZERO
        } else {
            T::from_str(repeating_part).map_err(|_| anyhow!("parsing failed"))?
        };
        num *= &repeat_den;
        num += repeat_num;
        repeat_den
    } else {
        T::ONE
    };

    if num == T::ZERO {
        return Ok(ParseFractionResult::Zero(sign));
    }

    if exp > 0 {
        num *= &T::TEN.pow(exp as u64);
    } else if exp < 0 {
        den *= &T::TEN.pow((-exp) as u64);
    }

    Ok(ParseFractionResult::Rational(sign, num, den))
}

fn parse_decimal_notation<T, E>(src: &str) -> Result<ParseFractionResult<T>, anyhow::Error>
where
    T: Clone
        + FromStr<Err = E>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + for<'a> MulAssign<&'a T>
        + AddAssign
        + PartialEq
        + Pow<u64, Output = T>
        + Zero
        + One
        + Ten,
{
    Ok(fraction_from_decimal(split_decimal_notation(src)?)?)
}

fn parse_fraction<T, E>(src: &str) -> Result<ParseFractionResult<T>, anyhow::Error>
where
    T: Clone
        + FromStr<Err = E>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + for<'a> MulAssign<&'a T>
        + AddAssign
        + PartialEq
        + Pow<u64, Output = T>
        + Zero
        + One
        + Ten,
{
    use ParseFractionResult::*;
    if src.eq_ignore_ascii_case("inf") | src.eq_ignore_ascii_case("+inf") {
        return Ok(Inf(true));
    } else if src.eq_ignore_ascii_case("-inf") {
        return Ok(Inf(false));
    } else if src.eq_ignore_ascii_case("nan")
        | src.eq_ignore_ascii_case("+nan")
        | src.eq_ignore_ascii_case("-nan")
    {
        return Ok(NaN);
    }
    match src.find('/') {
        Some(idx) => {
            let num_src = &src[..idx];
            let den_src = &src[idx + 1..];
            let mut sign = true;

            let num_src = match num_src.chars().next() {
                Some('+') => &num_src[1..],
                Some('-') => {
                    sign = !sign;
                    &num_src[1..]
                }
                _ => &num_src[..],
            };
            let den_src = match den_src.chars().next() {
                Some('+') => &den_src[1..],
                Some('-') => {
                    sign = !sign;
                    &den_src[1..]
                }
                _ => &den_src[..],
            };

            let num = if num_src.is_empty() {
                T::ONE
            } else {
                T::from_str(num_src).map_err(|_| anyhow!("parsing failed"))?
            };
            let den = if den_src.is_empty() {
                T::ONE
            } else {
                T::from_str(den_src).map_err(|_| anyhow!("parsing failed"))?
            };

            if den == T::ZERO {
                if num == T::ZERO {
                    Ok(NaN)
                } else {
                    Ok(Inf(sign))
                }
            } else if num == T::ZERO {
                Ok(Zero(sign))
            } else {
                Ok(Rational(sign, num, den))
            }
        }
        None => parse_decimal_notation(src),
    }
}

impl<T, E> FromStr for ParseFractionResult<T>
where
    T: Clone
        + FromStr<Err = E>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + for<'a> MulAssign<&'a T>
        + AddAssign
        + PartialEq
        + Pow<u64, Output = T>
        + Zero
        + One
        + Ten,
{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_fraction(s)
    }
}
