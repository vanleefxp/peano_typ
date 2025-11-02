use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::iter::{Product, Sum};
use std::ops::{Add, AddAssign, Deref, Div, Mul, MulAssign, Neg, Sub};
use std::{num::ParseIntError, str::FromStr};

use anyhow::{Context, anyhow, bail};
use fraction::{ConstOne, Ratio};
use fraction::{GenericFraction, generic::GenericInteger};
use malachite::base::num::arithmetic::traits::{
    Abs, AbsAssign, NegAssign, Pow, PowAssign, Reciprocal, ReciprocalAssign, Sign,
};
use malachite::base::num::basic::traits::{
    Infinity, NaN, NegativeInfinity, NegativeOne, NegativeZero, One, OneHalf, Two, Zero,
};
use malachite::rational::arithmetic::traits::{Approximate, ApproximateAssign};
use malachite::{Integer as Mpz, Natural as Mpn, Rational as Mpq};
use num::integer::Integer;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct FracData<T>
where
    T: Integer + Clone + Copy,
{
    pub sign: bool,
    pub num: T,
    pub den: T,
}

pub trait Ten {
    const TEN: Self;
}

pub trait Approx<N> {
    type Output;
    fn approx(self, max_den: &N) -> Self::Output;
}

#[allow(unused)]
pub trait ApproxAssign<N> {
    fn approx_assign(&mut self, max_den: &N);
}

impl Ten for Mpn {
    const TEN: Self = Mpn::const_from(10);
}

impl Ten for Mpz {
    const TEN: Self = Mpz::const_from_unsigned(10);
}

macro_rules! impl_10_for_primitives {
    ($($t: ty),*$(,)?) => {
        $(impl Ten for $t {
            const TEN: Self = 10 as $t;
        })*
    };
}
impl_10_for_primitives!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

pub struct Frac<T>(fraction::GenericFraction<T>)
where
    T: Clone + Integer;

impl<T> From<fraction::GenericFraction<T>> for Frac<T>
where
    T: Clone + Integer,
{
    fn from(value: fraction::GenericFraction<T>) -> Self {
        Frac(value)
    }
}

impl<T> From<Frac<T>> for GenericFraction<T>
where
    T: Clone + Integer,
{
    fn from(value: Frac<T>) -> Self {
        value.0
    }
}

impl<T> Deref for Frac<T>
where
    T: Clone + fraction::Integer,
{
    type Target = GenericFraction<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<GenericFraction<T>> for FracData<T>
where
    T: Integer + Clone + Copy,
{
    fn from(value: GenericFraction<T>) -> Self {
        use fraction::Sign::*;
        match value {
            GenericFraction::Rational(sign, ratio) => {
                let (num, den) = ratio.into_raw();
                FracData {
                    sign: sign == Plus,
                    num,
                    den,
                }
            }
            GenericFraction::Infinity(sign) => FracData {
                sign: sign == Plus,
                num: T::one(),
                den: T::zero(),
            },
            GenericFraction::NaN => FracData {
                sign: true,
                num: T::zero(),
                den: T::zero(),
            },
        }
    }
}

impl<T> Into<GenericFraction<T>> for FracData<T>
where
    T: Integer + Clone + Copy,
{
    fn into(self) -> GenericFraction<T> {
        use GenericFraction::*;
        use fraction::Sign::*;
        if self.den == T::zero() {
            if self.num == T::zero() {
                NaN
            } else {
                let sign = if self.sign { Plus } else { Minus };
                Infinity(sign)
            }
        } else {
            let sign = if self.sign { Plus } else { Minus };
            GenericFraction::new_raw_signed(sign, self.num, self.den)
        }
    }
}

impl<T> Into<Frac<T>> for FracData<T>
where
    T: Integer + Clone + Copy,
{
    fn into(self) -> Frac<T> {
        <FracData<T> as Into<GenericFraction<T>>>::into(self).into()
    }
}

fn limit_den_helper<T>((num, den): (T, T), max_den: T) -> Result<(T, T), String>
where
    T: Integer + Clone + Copy + Zero + One + Sign,
{
    if max_den.sign().is_gt() {
        let (mut p0, mut q0, mut p1, mut q1) = (T::ZERO, T::ONE, T::ONE, T::ZERO);
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

// fn limit_den_helper<T>(num: &T, den: &T, max_den: &T) -> Result<(T, T), String>
// where
//     T: Clone + Zero + One + Sign_ + Add<Output = T> + Sub<Output = T> + Mul<Output = T>  + Div<Output = T>,
//     for<'a> &'a T: Ord + Sub<T, Output = T> + Mul<&'a T, Output = T>,
// {
//     if max_den.sign().is_ge() {
//         let (mut p0, mut q0, mut p1, mut q1) = (T::ZERO, T::ONE, T::ONE, T::ZERO);
//         let (mut n, mut d) = (num.clone(), den.clone());
//         loop {
//             let a = n / d;
//             let q2 = q0 + &a * &q1;
//             if &q2 > max_den {
//                 break;
//             }
//             let (new_p0, new_q0) = (p1, q1);
//             let (new_p1, new_q1) = (p0 + a * p1, q2);
//             (p1, q1) = (new_p1, new_q1);
//             (p0, q0) = (new_p0, new_q0);
//             (n, d) = (d, n - a * d)
//         }
//         let k = (max_den - q0) / q1;
//         if &((d + d) * (q0 + k * q1)) <= den {
//             Ok((p1, q1))
//         } else {
//             Ok((p0 + k * p1, q0 + k * q1))
//         }
//     } else {
//         Err("max_den must be positive".to_string())
//     }
// }

// impl<T> Frac<T>
// where
//     T: Integer + Clone + Copy,
// {
//     pub fn limit_den(self: Frac<T>, max_den: T) -> Frac<T> {
//         match self.0 {
//             GenericFraction::Rational(sign, ratio) => {
//                 let (num, den) = ratio.into_raw();
//                 let (num, den) = limit_den_helper((num, den), max_den).unwrap();
//                 GenericFraction::Rational(sign, Ratio::new_raw(num, den)).into()
//             }
//             special => special.into(),
//         }
//     }
// }

impl<T> Approx<T> for Frac<T>
where
    T: Integer + Clone + Copy + Zero + One + Sign,
{
    type Output = Self;

    fn approx(self, max_den: &T) -> Self::Output {
        use GenericFraction::*;
        match self.0 {
            Rational(sign, ratio) => {
                let (num, den) = ratio.into_raw();
                let (num, den) = limit_den_helper((num, den), *max_den).unwrap();
                Rational(sign, Ratio::new_raw(num, den)).into()
            }
            special => special.into(),
        }
    }
}

impl Pow<i64> for Frac<u64> {
    type Output = Self;

    fn pow(self, rhs: i64) -> Self::Output {
        use fraction::GenericFraction::*;
        use fraction::Sign::*;
        if rhs == 0 {
            GenericFraction::ONE.into()
        } else {
            match self.0 {
                Rational(sign, ratio) => {
                    let (num, den) = ratio.into_raw();
                    let sign = match sign {
                        Plus => Plus,
                        Minus => {
                            if rhs % 2 == 0 {
                                Plus
                            } else {
                                Minus
                            }
                        }
                    };
                    if rhs > 0 {
                        let (num, den) = (num.pow(rhs as u32), den.pow(rhs as u32));
                        Rational(sign, Ratio::new_raw(num, den)).into()
                    } else {
                        let (num, den) = (den.pow((-rhs) as u32), num.pow((-rhs) as u32));
                        Rational(sign, Ratio::new_raw(num, den)).into()
                    }
                }
                NaN | Infinity(Plus) => self,
                Infinity(Minus) => {
                    if rhs % 2 == 1 {
                        self
                    } else {
                        Infinity(Plus).into()
                    }
                }
            }
        }
    }
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

enum ParseFractionResult<T> {
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
    if src.eq_ignore_ascii_case("inf") | src.eq_ignore_ascii_case("+inf") {
        return Ok(ParseFractionResult::Inf(true));
    } else if src.eq_ignore_ascii_case("-inf") {
        return Ok(ParseFractionResult::Inf(false));
    } else if src.eq_ignore_ascii_case("nan") {
        return Ok(ParseFractionResult::NaN);
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
                    Ok(ParseFractionResult::NaN)
                } else {
                    Ok(ParseFractionResult::Inf(sign))
                }
            } else if num == T::ZERO {
                Ok(ParseFractionResult::Zero(sign))
            } else {
                Ok(ParseFractionResult::Rational(sign, num, den))
            }
        }
        None => parse_decimal_notation(src),
    }
}

impl<T> From<ParseFractionResult<T>> for Frac<T>
where
    T: Clone + fraction::Integer + Zero + One,
{
    fn from(value: ParseFractionResult<T>) -> Self {
        use ParseFractionResult::*;
        use fraction::Sign::*;
        match value {
            Rational(s, num, den) => {
                GenericFraction::Rational(if s { Plus } else { Minus }, Ratio::new(num, den))
            }
            Inf(s) => GenericFraction::Infinity(if s { Plus } else { Minus }).into(),
            Zero(s) => GenericFraction::Rational(
                if s { Plus } else { Minus },
                Ratio::new_raw(T::ZERO, T::ONE),
            ),
            NaN => GenericFraction::NaN,
        }
        .into()
    }
}

impl<T> FromStr for Frac<T>
where
    T: GenericInteger
        + Clone
        + Copy
        + FromStr<Err = ParseIntError>
        + Pow<u64, Output = T>
        + Into<GenericFraction<T>>
        + fraction::Integer
        + Zero
        + One
        + Ten,
{
    type Err = anyhow::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(parse_fraction(src)?.into())
    }
}

// Extending malachite::Rational with infinity and NaN support

#[derive(Clone, Serialize, Deserialize, Hash)]
pub enum MpqExt {
    Zero(bool),
    Inf(bool),
    NaN,
    Rational(Mpq),
}

pub trait SignStrict {
    fn sign_strict(&self) -> Ordering;
}

#[allow(unused)]
pub trait ExtendedNumber: Sign + SignStrict {
    fn is_nan(&self) -> bool;
    fn is_infinite(&self) -> bool;
    fn is_zero(&self) -> bool {
        self.sign().is_eq()
    }
    fn is_finite(&self) -> bool {
        !self.is_nan() & !self.is_infinite()
    }
    fn is_sign_positive(&self) -> bool {
        !self.is_nan() & self.sign_strict().is_gt()
    }
    fn is_sign_negative(&self) -> bool {
        !self.is_sign_positive() & !self.is_nan()
    }
}

impl<T> SignStrict for T
where
    T: num::Float + Zero,
{
    fn sign_strict(&self) -> Ordering {
        use Ordering::*;
        match self.partial_cmp(&Self::ZERO) {
            Some(ordering) => ordering,
            None => Equal,
        }
    }
}

impl<T> ExtendedNumber for T
where
    T: num::Float + Sign + SignStrict,
{
    fn is_nan(&self) -> bool {
        num::Float::is_nan(*self)
    }

    fn is_finite(&self) -> bool {
        num::Float::is_finite(*self)
    }

    fn is_infinite(&self) -> bool {
        num::Float::is_infinite(*self)
    }

    fn is_sign_positive(&self) -> bool {
        num::Float::is_sign_positive(*self)
    }

    fn is_sign_negative(&self) -> bool {
        num::Float::is_sign_negative(*self)
    }
}

impl MpqExt {
    #[inline]
    pub fn into_numerator(self) -> Mpn {
        use MpqExt::*;
        match self {
            Zero(_) | NaN => Mpn::ZERO,
            Inf(_) => Mpn::ONE,
            Rational(q) => q.into_numerator(),
        }
    }

    #[inline]
    pub fn into_numerator_signed(self) -> Mpz {
        use MpqExt::*;
        match self {
            Zero(_) | NaN => Mpz::ZERO,
            Inf(true) => Mpz::ONE,
            Inf(false) => Mpz::NEGATIVE_ONE,
            Rational(q) => Mpz::from_sign_and_abs(q >= 0, q.into_numerator()),
        }
    }

    #[inline]
    pub fn into_denominator(self) -> Mpn {
        use MpqExt::*;
        match self {
            Zero(_) | NaN => Mpn::ONE,
            Inf(_) => Mpn::ZERO,
            Rational(q) => q.into_denominator(),
        }
    }

    #[inline]
    pub fn into_denominator_signed(self) -> Mpz {
        use MpqExt::*;
        match self {
            NaN | Zero(true) => Mpz::ONE,
            Zero(false) => Mpz::NEGATIVE_ONE,
            Inf(_) => Mpz::ZERO,
            Rational(q) => Mpz::from_sign_and_abs(q >= 0, q.into_numerator()),
        }
    }

    #[inline]
    pub fn into_numerator_and_denominator(self) -> (Mpn, Mpn) {
        use MpqExt::*;
        match self {
            Zero(_) => (Mpn::ZERO, Mpn::ONE),
            NaN => (Mpn::ZERO, Mpn::ZERO),
            Inf(_) => (Mpn::ONE, Mpn::ZERO),
            Rational(q) => q.into_numerator_and_denominator(),
        }
    }

    #[inline]
    pub fn to_numerator(&self) -> Mpn {
        use MpqExt::*;
        match self {
            Zero(_) | NaN => Mpn::ZERO,
            Inf(_) => Mpn::ONE,
            Rational(q) => q.to_numerator(),
        }
    }

    #[inline]
    pub fn to_denominator(&self) -> Mpn {
        use MpqExt::*;
        match self {
            Zero(_) | NaN => Mpn::ONE,
            Inf(_) => Mpn::ZERO,
            Rational(q) => q.to_denominator(),
        }
    }

    // `numerator_ref` and `denominator_ref` are not possible

    #[inline]
    pub fn to_numerator_and_denominator(&self) -> (Mpn, Mpn) {
        use MpqExt::*;
        match self {
            Zero(_) => (Mpn::ZERO, Mpn::ONE),
            NaN => (Mpn::ZERO, Mpn::ZERO),
            Inf(_) => (Mpn::ONE, Mpn::ZERO),
            Rational(q) => q.to_numerator_and_denominator(),
        }
    }

    pub fn from_sign_and_naturals(sign: bool, n: Mpn, d: Mpn) -> Self {
        match (n, d) {
            (Mpn::ZERO, Mpn::ZERO) => Self::NaN,
            (Mpn::ZERO, _) => Self::Zero(sign),
            (_, Mpn::ZERO) => Self::Inf(sign),
            (n, d) => Self::Rational(Mpq::from_sign_and_naturals(sign, n, d)),
        }
    }

    pub fn from_sign_and_naturals_ref(sign: bool, n: &Mpn, d: &Mpn) -> Self {
        match (n, d) {
            (&Mpn::ZERO, &Mpn::ZERO) => Self::NaN,
            (&Mpn::ZERO, _) => Self::Zero(sign),
            (_, &Mpn::ZERO) => Self::Inf(sign),
            (n, d) => Self::Rational(Mpq::from_sign_and_naturals_ref(sign, n, d)),
        }
    }

    pub fn from_integers(n: Mpz, d: Mpz) -> Self {
        match (n, d) {
            (Mpz::ZERO, Mpz::ZERO) => Self::NaN,
            (Mpz::ZERO, d) => Self::Zero(d >= 0),
            (n, Mpz::ZERO) => Self::Inf(n >= 0),
            (n, d) => Self::Rational(Mpq::from_integers(n, d)),
        }
    }

    pub fn from_integers_ref(n: &Mpz, d: &Mpz) -> Self {
        match (n, d) {
            (&Mpz::ZERO, &Mpz::ZERO) => Self::NaN,
            (&Mpz::ZERO, d) => Self::Zero(d >= &0),
            (n, &Mpz::ZERO) => Self::Inf(n >= &0),
            (n, d) => Self::Rational(Mpq::from_integers_ref(n, d)),
        }
    }
}

impl Sign for MpqExt {
    fn sign(&self) -> Ordering {
        use MpqExt::*;
        use Ordering::*;
        match self {
            Zero(_) | NaN => Equal,
            &Inf(s) => {
                if s {
                    Greater
                } else {
                    Less
                }
            }
            Rational(q) => q.sign(),
        }
    }
}

impl SignStrict for MpqExt {
    fn sign_strict(&self) -> Ordering {
        use MpqExt::*;
        use Ordering::*;
        match self {
            NaN => Equal,
            &Zero(s) | &Inf(s) => {
                if s {
                    Greater
                } else {
                    Less
                }
            }
            Rational(q) => q.sign(),
        }
    }
}

impl ExtendedNumber for MpqExt {
    #[inline]
    fn is_nan(&self) -> bool {
        use MpqExt::*;
        match self {
            NaN => true,
            _ => false,
        }
    }

    #[inline]
    fn is_zero(&self) -> bool {
        match self {
            Self::Zero(_) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_infinite(&self) -> bool {
        use MpqExt::*;
        match self {
            Inf(_) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_finite(&self) -> bool {
        use MpqExt::*;
        match self {
            Inf(_) | NaN => false,
            _ => true,
        }
    }

    #[inline]
    fn is_sign_positive(&self) -> bool {
        use MpqExt::*;
        match self {
            NaN => false,
            &Zero(s) | &Inf(s) => s,
            Rational(q) => q.sign().is_gt(),
        }
    }

    #[inline]
    fn is_sign_negative(&self) -> bool {
        use MpqExt::*;
        match self {
            NaN => false,
            &Zero(s) | &Inf(s) => !s,
            Rational(q) => q.sign().is_lt(),
        }
    }
}

impl From<ParseFractionResult<Mpn>> for MpqExt {
    fn from(value: ParseFractionResult<Mpn>) -> Self {
        use ParseFractionResult::*;
        match value {
            NaN => Self::NaN,
            Zero(s) => Self::Zero(s),
            Inf(s) => Self::Inf(s),
            Rational(s, n, d) => Self::Rational(Mpq::from_sign_and_naturals(s, n, d)),
        }
    }
}

impl FromStr for MpqExt {
    type Err = anyhow::Error;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(parse_fraction(src)?.into())
    }
}

impl From<Mpz> for MpqExt {
    fn from(value: Mpz) -> Self {
        match value {
            Mpz::ZERO => Self::Zero(true),
            _ => Self::Rational(Mpq::from(value)),
        }
    }
}

impl From<Mpn> for MpqExt {
    fn from(value: Mpn) -> Self {
        match value {
            Mpn::ZERO => Self::Zero(true),
            _ => Self::Rational(Mpq::from(value)),
        }
    }
}

macro_rules! impl_mpq_ext_from_int {
    ($($t:ty),+$(,)?) => {
        $(impl From<$t> for MpqExt {
            fn from(value: $t) -> Self {
                match value {
                    0 => Self::Zero(true),
                    _ => Self::Rational(Mpq::from(value)),
                }
            }
        })*
    };
}

macro_rules! impl_mpq_ext_try_from_float {
    ($($t:ty),+$(,)?) => {
        $(impl TryFrom<$t> for MpqExt {
            type Error = anyhow::Error;
            fn try_from(value: $t) -> Result<Self, Self::Error> {
                use MpqExt::*;
                if value.is_nan() {
                    Ok(NaN)
                } else if value == <$t>::ZERO {
                    Ok(Zero(value.is_sign_positive()))
                } else if value.is_infinite() {
                    Ok(Inf(value.is_sign_positive()))
                } else {
                    Ok(Rational(Mpq::try_from(value).map_err(|_| anyhow!("parse failed"))?))
                }
            }
        })*
    }
}

impl_mpq_ext_from_int!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_mpq_ext_try_from_float!(/*f16,*/ f32, f64 /*f128*/,);

impl From<Mpq> for MpqExt {
    fn from(value: Mpq) -> Self {
        match value {
            Mpq::ZERO => Self::Zero(true),
            _ => Self::Rational(value),
        }
    }
}

impl From<&Mpq> for MpqExt {
    fn from(value: &Mpq) -> Self {
        match value {
            &Mpq::ZERO => Self::Zero(true),
            _ => Self::Rational(value.clone()),
        }
    }
}

impl TryInto<Mpq> for MpqExt {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Mpq, Self::Error> {
        match self {
            MpqExt::Zero(_) => Ok(Mpq::ZERO),
            MpqExt::Rational(q) => Ok(q),
            _ => Err(anyhow!("infinity or NaN cannot be converted")),
        }
    }
}

impl One for MpqExt {
    const ONE: Self = Self::Rational(Mpq::ONE);
}

impl Zero for MpqExt {
    const ZERO: Self = Self::Zero(true);
}

impl NegativeZero for MpqExt {
    const NEGATIVE_ZERO: Self = Self::Zero(false);
}

impl NegativeOne for MpqExt {
    const NEGATIVE_ONE: Self = Self::Rational(Mpq::NEGATIVE_ONE);
}

impl Two for MpqExt {
    const TWO: Self = Self::Rational(Mpq::TWO);
}

impl OneHalf for MpqExt {
    const ONE_HALF: Self = Self::Rational(Mpq::ONE_HALF);
}

impl Infinity for MpqExt {
    const INFINITY: Self = Self::Inf(true);
}

impl NegativeInfinity for MpqExt {
    const NEGATIVE_INFINITY: Self = Self::Inf(false);
}

impl NaN for MpqExt {
    const NAN: Self = Self::NaN;
}

impl Default for MpqExt {
    fn default() -> Self {
        Self::ZERO
    }
}

macro_rules! impl_neg_for_mpq_ext {
    ($($t:ty),+$(,)?) => {
        $(impl Neg for $t {
            type Output = MpqExt;

            fn neg(self) -> Self::Output {
                use MpqExt::*;
                match self {
                    Rational(q) => Rational(-q),
                    Inf(s) => Inf(!s),
                    Zero(s) => Zero(!s),
                    NaN => NaN,
                }
            }
        })*
    };
}
impl_neg_for_mpq_ext!(MpqExt, &MpqExt);

impl NegAssign for MpqExt {
    fn neg_assign(&mut self) {
        use MpqExt::*;
        match self {
            Zero(s) | Inf(s) => *s = !*s,
            NaN => {}
            Rational(q) => q.neg_assign(),
        }
    }
}

impl Reciprocal for MpqExt {
    type Output = Self;

    fn reciprocal(self) -> Self::Output {
        use MpqExt::*;
        match self {
            Rational(q) => Rational(q.reciprocal()),
            Inf(s) => Zero(s),
            Zero(s) => Inf(s),
            NaN => NaN,
        }
    }
}

impl Reciprocal for &MpqExt {
    type Output = MpqExt;

    fn reciprocal(self) -> Self::Output {
        use MpqExt::*;
        match self {
            Rational(q) => Rational(q.reciprocal()),
            Inf(s) => Zero(*s),
            Zero(s) => Inf(*s),
            NaN => NaN,
        }
    }
}

impl ReciprocalAssign for MpqExt {
    fn reciprocal_assign(&mut self) {
        use MpqExt::*;
        match self {
            Rational(q) => q.reciprocal_assign(),
            Inf(s) => *self = Zero(*s),
            Zero(s) => *self = Inf(*s),
            NaN => {}
        }
    }
}

impl Add for MpqExt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use MpqExt::*;
        match (self, rhs) {
            (_, NaN) | (NaN, _) => NaN,
            (Inf(s1), Inf(s2)) if s1 != s2 => NaN,
            (Zero(s1), Zero(s2)) => Zero(s1 || s2),
            (Zero(_), r) | (r, Zero(_)) => r,
            (Inf(s), _) | (_, Inf(s)) => Inf(s),
            (Rational(q1), Rational(q2)) => (q1 + q2).into(),
        }
    }
}

impl Add<&Self> for MpqExt {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        use MpqExt::*;
        match (self, rhs) {
            (_, NaN) | (NaN, _) => NaN,
            (Inf(s1), &Inf(s2)) if s1 != s2 => NaN,
            (Zero(s1), &Zero(s2)) => Zero(s1 || s2),
            (Zero(_), r) => r.clone(),
            (r, Zero(_)) => r,
            (Inf(s), _) | (_, &Inf(s)) => Inf(s),
            (Rational(q1), Rational(q2)) => (q1 + q2).into(),
        }
    }
}

impl Add<MpqExt> for &MpqExt {
    type Output = MpqExt;

    fn add(self, rhs: MpqExt) -> Self::Output {
        rhs + self
    }
}

impl Add for &MpqExt {
    type Output = MpqExt;
    fn add(self, rhs: Self) -> Self::Output {
        use MpqExt::*;
        match (self, rhs) {
            (_, NaN) | (NaN, _) => NaN,
            (Inf(s1), Inf(s2)) if s1 != s2 => NaN,
            (&Zero(s1), &Zero(s2)) => Zero(s1 || s2),
            (Zero(_), r) | (r, Zero(_)) => r.clone(),
            (&Inf(s), _) | (_, &Inf(s)) => Inf(s),
            (Rational(q1), Rational(q2)) => (q1 + q2).into(),
        }
    }
}

impl AddAssign for MpqExt {
    fn add_assign(&mut self, rhs: Self) {
        use MpqExt::*;
        *self = match (std::mem::replace(self, NaN), rhs) {
            (NaN, _) | (_, NaN) => NaN,
            (Inf(s1), Inf(s2)) if s1 != s2 => NaN,
            (Inf(s), _) | (_, Inf(s)) => Inf(s),
            (Zero(s1), Zero(s2)) => Zero(s1 || s2),
            (Zero(_), r) | (r, Zero(_)) => r,
            (Rational(mut q1), Rational(q2)) => {
                q1 += q2;
                if q1 == 0 { Zero(true) } else { Rational(q1) }
            }
        }
        // match (self, rhs) {
        //     (NaN, _)
        //     | (Zero(_), Zero(false))
        //     | (Rational(_), Zero(_))
        //     | (Inf(_), Zero(_) | Rational(_))
        //     | (Inf(true), Inf(true))
        //     | (Inf(false), Inf(false)) => {}
        //     (a, NaN)
        //     | (a @ Inf(true), Inf(false))
        //     | (a @ Inf(false), Inf(true)) => *a = MpqExt::NaN,
        //     (Zero(s), Zero(true)) => *s = true,
        //     (a @ Zero(_), other @ (Rational(_) | Inf(_)))
        //     | (a @ Rational(_), other @ Inf(_)) => *a = other,
        //     (a @ Rational(_), Rational(q2)) => {
        //         if let Rational(q1) = a {
        //             *q1 += q2;
        //             if q1 == &0 {
        //                 *a = MpqExt::ZERO;
        //             }
        //         } else {
        //             unreachable!();
        //         }
        //     },
        // }
    }
}

impl AddAssign<&Self> for MpqExt {
    fn add_assign(&mut self, rhs: &Self) {
        use MpqExt::*;
        *self = match (std::mem::replace(self, NaN), rhs) {
            (NaN, _) | (_, NaN) => NaN,
            (Inf(s1), &Inf(s2)) if s1 != s2 => NaN,
            (Inf(s), _) | (_, &Inf(s)) => Inf(s),
            (Zero(s1), &Zero(s2)) => Zero(s1 || s2),
            (Zero(_), r) => r.clone(),
            (r, Zero(_)) => r,
            (Rational(mut q1), Rational(q2)) => {
                q1 += q2;
                if q1 == 0 { Zero(true) } else { Rational(q1) }
            }
        }
        // match (self, rhs) {
        //     (NaN, _)
        //     | (Zero(_), Zero(false))
        //     | (Rational(_), Zero(_))
        //     | (Inf(_), Zero(_) | Rational(_))
        //     | (Inf(true), Inf(true))
        //     | (Inf(false), Inf(false)) => {}
        //     (a, NaN)
        //     | (a @ Inf(true), Self::Inf(false))
        //     | (a @ Inf(false), Self::Inf(true)) => *a = MpqExt::NaN,
        //     (Zero(s), Zero(true)) => *s = true,
        //     (a @ Zero(_), other @ (Rational(_) | Inf(_)))
        //     | (a @ Self::Rational(_), other @ Inf(_)) => *a = other.clone(),
        //     (a @ Rational(_), Rational(q2)) => {
        //         if let Rational(q1) = a {
        //             *q1 += q2;
        //             if q1 == &0 {
        //                 *a = MpqExt::ZERO;
        //             }
        //         } else {
        //             unreachable!();
        //         }
        //     },
        // }
    }
}

impl Sum for MpqExt {
    fn sum<I>(xs: I) -> MpqExt
    where
        I: Iterator<Item = MpqExt>,
    {
        let mut stack = Vec::new();
        for (i, x) in xs.enumerate() {
            if x.is_nan() {
                return MpqExt::NaN;
            }
            let mut s = x;
            for _ in 0..(i + 1).trailing_zeros() {
                s += stack.pop().unwrap();
            }
            stack.push(s);
        }
        let mut s = MpqExt::ZERO;
        for x in stack.into_iter().rev() {
            s += x;
        }
        s
    }
}

impl<'a> Sum<&'a MpqExt> for MpqExt {
    fn sum<I>(xs: I) -> MpqExt
    where
        I: Iterator<Item = &'a MpqExt>,
    {
        let mut stack = Vec::new();
        for (i, x) in xs.enumerate() {
            if x.is_nan() {
                return MpqExt::NaN;
            }
            let mut s = x.clone();
            for _ in 0..(i + 1).trailing_zeros() {
                s += stack.pop().unwrap();
            }
            stack.push(s);
        }
        let mut s = MpqExt::ZERO;
        for x in stack.into_iter().rev() {
            s += x;
        }
        s
    }
}

impl Mul for MpqExt {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        use MpqExt::*;
        match (self, rhs) {
            (NaN, _) | (_, NaN) | (Zero(_), Inf(_)) | (Inf(_), Zero(_)) => NaN,
            (Zero(s1), Zero(s2)) => Zero(s1 == s2),
            (Zero(s1), Rational(q)) | (Rational(q), Zero(s1)) => {
                let s2 = q >= 0;
                Zero(s1 == s2)
            }
            (Inf(s1), Inf(s2)) => Inf(s1 == s2),
            (Inf(s1), Rational(q)) | (Rational(q), Inf(s1)) => {
                let s2 = q >= 0;
                Inf(s1 == s2)
            }
            (Rational(q1), Rational(q2)) => Rational(q1 * q2),
        }
    }
}

impl Mul<&Self> for MpqExt {
    type Output = Self;
    fn mul(self, rhs: &Self) -> Self::Output {
        use MpqExt::*;
        match (self, rhs) {
            (NaN, _) | (_, NaN) | (Zero(_), Inf(_)) | (Inf(_), Zero(_)) => NaN,
            (Zero(s1), &Zero(s2)) => Zero(s1 == s2),
            (Zero(s1), Rational(q)) => {
                let s2 = q >= &0;
                Zero(s1 == s2)
            }
            (Rational(q), &Zero(s1)) => {
                let s2 = q >= 0;
                Zero(s1 == s2)
            }
            (Inf(s1), &Inf(s2)) => Inf(s1 == s2),
            (Inf(s1), Rational(q)) => {
                let s2 = q >= &0;
                Inf(s1 == s2)
            }
            (Rational(q), &Inf(s1)) => {
                let s2 = q >= 0;
                Inf(s1 == s2)
            }
            (Rational(q1), Rational(q2)) => Rational(q1 * q2),
        }
    }
}

impl Mul<MpqExt> for &MpqExt {
    type Output = MpqExt;

    fn mul(self, rhs: MpqExt) -> Self::Output {
        rhs * self
    }
}

impl Mul<Self> for &MpqExt {
    type Output = MpqExt;
    fn mul(self, rhs: Self) -> Self::Output {
        use MpqExt::*;
        match (self, rhs) {
            (NaN, _) | (_, NaN) | (Zero(_), Inf(_)) | (Inf(_), Zero(_)) => NaN,
            (Zero(s1), Zero(s2)) => Zero(s1 == s2),
            (&Zero(s1), Rational(q)) | (Rational(q), &Zero(s1)) => {
                let s2 = q >= &0;
                Zero(s1 == s2)
            }
            (Inf(s1), Inf(s2)) => Inf(s1 == s2),
            (&Inf(s1), Rational(q)) | (Rational(q), &Inf(s1)) => {
                let s2 = q >= &0;
                Inf(s1 == s2)
            }
            (Rational(q1), Rational(q2)) => Rational(q1 * q2),
        }
    }
}

impl MulAssign for MpqExt {
    fn mul_assign(&mut self, rhs: Self) {
        use MpqExt::*;
        match rhs {
            Rational(Mpq::ONE) => {}
            _ => {
                let temp = std::mem::replace(self, NaN);
                *self = temp * rhs;
            }
        }
    }
}

impl MulAssign<&MpqExt> for MpqExt {
    fn mul_assign(&mut self, rhs: &Self) {
        use MpqExt::*;
        match rhs {
            Rational(Mpq::ONE) => {}
            _ => {
                let temp = std::mem::replace(self, NaN);
                *self = temp * rhs;
            }
        }
    }
}

impl Product for MpqExt {
    fn product<I>(xs: I) -> MpqExt
    where
        I: Iterator<Item = MpqExt>,
    {
        use MpqExt::*;
        let mut stack = Vec::new();
        for (i, x) in xs.enumerate() {
            if x.is_nan() {
                return NaN;
            }
            let mut s = x;
            for _ in 0..(i + 1).trailing_zeros() {
                s *= stack.pop().unwrap();
            }
            stack.push(s);
        }
        let mut s = MpqExt::ONE;
        for x in stack.into_iter().rev() {
            s *= x;
        }
        s
    }
}

impl<'a> Product<&'a MpqExt> for MpqExt {
    fn product<I>(xs: I) -> MpqExt
    where
        I: Iterator<Item = &'a MpqExt>,
    {
        use MpqExt::*;
        let mut stack = Vec::new();
        for (i, x) in xs.enumerate() {
            if x.is_nan() {
                return NaN;
            }
            let mut s = x.clone();
            for _ in 0..(i + 1).trailing_zeros() {
                s *= stack.pop().unwrap();
            }
            stack.push(s);
        }
        let mut s = MpqExt::ONE;
        for x in stack.into_iter().rev() {
            s *= x;
        }
        s
    }
}

impl Sub for MpqExt {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        use MpqExt::*;
        match (self, rhs) {
            (_, NaN) | (NaN, _) => NaN,
            (Inf(s1), Inf(s2)) if s1 == s2 => NaN,
            (Zero(s1), Zero(s2)) => Zero(s1 || !s2),
            (other, Zero(_)) => other,
            (Zero(_), other) => -other,
            (Inf(true), _) | (_, Inf(false)) => Self::INFINITY,
            (Inf(false), _) | (_, Inf(true)) => Self::NEGATIVE_INFINITY,
            (Rational(q1), Rational(q2)) => Rational(q1 - q2),
        }
    }
}

impl Sub<&Self> for MpqExt {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self::Output {
        use MpqExt::*;
        match (self, rhs) {
            (_, NaN) | (NaN, _) => NaN,
            (Inf(s1), &Inf(s2)) if s1 == s2 => NaN,
            (Zero(s1), &Zero(s2)) => Zero(s1 || !s2),
            (other, Zero(_)) => other,
            (Zero(_), other) => -other,
            (Inf(true), _) | (_, Inf(false)) => Self::INFINITY,
            (Inf(false), _) | (_, Inf(true)) => Self::NEGATIVE_INFINITY,
            (Rational(q1), Rational(q2)) => Rational(q1 - q2),
        }
    }
}

impl Sub<MpqExt> for &MpqExt {
    type Output = MpqExt;

    fn sub(self, rhs: MpqExt) -> Self::Output {
        use MpqExt::*;
        match (self, rhs) {
            (_, NaN) | (NaN, _) => NaN,
            (&Inf(s1), Inf(s2)) if s1 == s2 => NaN,
            (&Zero(s1), Zero(s2)) => Zero(s1 || !s2),
            (other, Zero(_)) => other.clone(),
            (Zero(_), other) => -other,
            (Inf(true), _) | (_, Inf(false)) => MpqExt::INFINITY,
            (Inf(false), _) | (_, Inf(true)) => MpqExt::NEGATIVE_INFINITY,
            (Rational(q1), Rational(q2)) => Rational(q1 - q2),
        }
    }
}

impl Sub<Self> for &MpqExt {
    type Output = MpqExt;

    fn sub(self, rhs: Self) -> Self::Output {
        use MpqExt::*;
        match (self, rhs) {
            (_, NaN) | (NaN, _) => NaN,
            (&Inf(s1), &Inf(s2)) if s1 == s2 => NaN,
            (&Zero(s1), &Zero(s2)) => Zero(s1 || !s2),
            (other, Zero(_)) => other.clone(),
            (Zero(_), other) => -other,
            (Inf(true), _) | (_, Inf(false)) => MpqExt::INFINITY,
            (Inf(false), _) | (_, Inf(true)) => MpqExt::NEGATIVE_INFINITY,
            (Rational(q1), Rational(q2)) => Rational(q1 - q2),
        }
    }
}

impl Div for MpqExt {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::NaN, _)
            | (_, Self::NaN)
            | (Self::Zero(_), Self::Zero(_))
            | (Self::Inf(_), Self::Inf(_)) => Self::NaN,
            (Self::Zero(s1), Self::Inf(s2)) => Self::Zero(s1 == s2),
            (Self::Inf(s1), Self::Zero(s2)) => Self::Inf(s1 == s2),
            (Self::Inf(s1), Self::Rational(q)) | (Self::Rational(q), Self::Zero(s1)) => {
                let s2 = q >= 0;
                Self::Inf(s1 == s2)
            }
            (Self::Rational(q), Self::Inf(s1)) | (Self::Zero(s1), Self::Rational(q)) => {
                let s2 = q >= 0;
                Self::Zero(s1 == s2)
            }
            (Self::Rational(q1), Self::Rational(q2)) => Self::Rational(q1 / q2),
        }
    }
}

impl Sub<Mpq> for MpqExt {
    type Output = Self;

    fn sub(self, rhs: Mpq) -> Self::Output {
        self - MpqExt::from(rhs)
    }
}

macro_rules! impl_pow_for_mpq_ext {
    ($($t:ty),+$(,)?) => {
        $(impl Pow<u64> for $t {
            type Output = MpqExt;

            fn pow(self, exp: u64) -> Self::Output {
                use MpqExt::*;
                match self {
                    NaN => NaN,
                    Zero(true) => if exp > 0 {
                        MpqExt::ZERO
                    } else {
                        MpqExt::ONE
                    },
                    Zero(false) => if exp > 0 {
                        Zero(exp % 2 == 1)
                    } else {
                        MpqExt::ONE
                    },
                    Inf(true) => if exp > 0 {
                        MpqExt::INFINITY
                    } else {
                        MpqExt::ONE
                    },
                    Inf(false) => if exp > 0 {
                        Inf(exp % 2 == 1)
                    } else {
                        MpqExt::ONE
                    },
                    Rational(q) => {
                        Rational(q.pow(exp))
                    }
                }
            }
        }

        impl Pow<i64> for $t {
            type Output = MpqExt;

            fn pow(self, exp: i64) -> Self::Output {
                use MpqExt::*;
                match self {
                    NaN => NaN,
                    Zero(true) => if exp > 0 {
                        MpqExt::ZERO
                    } else if exp == 0 {
                        MpqExt::ONE
                    } else {
                        MpqExt::INFINITY
                    },
                    Zero(false) => if exp > 0 {
                        Zero(exp % 2 == 1)
                    } else if exp == 0 {
                        MpqExt::ONE
                    } else {
                        Inf(exp % 2 == 1)
                    },
                    Inf(true) => if exp > 0 {
                        MpqExt::INFINITY
                    } else if exp == 0 {
                        MpqExt::ONE
                    } else {
                        MpqExt::ZERO
                    },
                    Inf(false) => if exp > 0 {
                        Inf(exp % 2 == 1)
                    } else if exp == 0 {
                        MpqExt::ONE
                    } else {
                        MpqExt::ZERO
                    },
                    Rational(q) => {
                        Rational(q.pow(exp))
                    }
                }
            }
        })*
    };
}

impl_pow_for_mpq_ext!(MpqExt, &MpqExt);

impl PowAssign<u64> for MpqExt {
    fn pow_assign(&mut self, exp: u64) {
        use MpqExt::*;
        *self = std::mem::replace(self, NaN).pow(exp);
    }
}

impl PowAssign<i64> for MpqExt {
    fn pow_assign(&mut self, exp: i64) {
        use MpqExt::*;
        *self = std::mem::replace(self, NaN).pow(exp);
    }
}

macro_rules! impl_abs_for_mpq_ext {
    ($($t:ty),*$(,)?) => {
        $(impl Abs for $t {
            type Output = MpqExt;
            fn abs(self) -> Self::Output {
                use MpqExt::*;
                match self {
                    NaN => NaN,
                    Zero(_) => Zero(true),
                    Inf(_) => Inf(true),
                    Rational(q) => Rational(q.abs()),
                }
            }
        })*
    };
}

impl_abs_for_mpq_ext!(MpqExt, &MpqExt);

impl AbsAssign for MpqExt {
    fn abs_assign(&mut self) {
        use MpqExt::*;
        match self {
            NaN => {}
            Zero(s) => *s = true,
            Inf(s) => *s = true,
            Rational(q) => (*q).abs_assign(),
        }
    }
}

impl Display for MpqExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MpqExt::*;
        match self {
            NaN => write!(f, "NaN"),
            Zero(true) => write!(f, "0"),
            Zero(false) => write!(f, "-0"),
            Inf(true) => write!(f, "inf"),
            Inf(false) => write!(f, "-inf"),
            Rational(q) => Display::fmt(q, f),
        }
    }
}

impl Debug for MpqExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <MpqExt as Display>::fmt(self, f)
    }
}

impl PartialOrd for MpqExt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use MpqExt::*;
        use Ordering::*;
        match (self, other) {
            (MpqExt::NaN, _) | (_, MpqExt::NaN) => None,
            (Inf(s1), Inf(s2)) if s1 == s2 => Some(Equal),
            (_, Inf(true)) | (Inf(false), _) => Some(Less),
            (Inf(true), _) | (_, Inf(false)) => Some(Greater),
            (Zero(_), Zero(_)) => Some(Equal),
            (Zero(_), Rational(q)) => Some(q.sign().reverse()),
            (Rational(q), Zero(_)) => Some(q.sign()),
            (Rational(q1), Rational(q2)) => q1.partial_cmp(q2),
        }
    }
}

impl PartialEq for MpqExt {
    fn eq(&self, other: &Self) -> bool {
        use MpqExt::*;
        match (self, other) {
            (Zero(_), Zero(_)) => true,
            (Inf(s1), Inf(s2)) => s1 == s2,
            (Rational(q1), Rational(q2)) => q1 == q2,
            _ => false,
        }
    }
}

impl MpqExt {
    pub fn partial_cmp_strict(&self, other: &Self) -> Option<Ordering> {
        use MpqExt::*;
        use Ordering::*;
        match (self, other) {
            (MpqExt::NaN, _) | (_, MpqExt::NaN) => None,
            (Inf(s1), Inf(s2)) if s1 == s2 => Some(Equal),
            (_, Inf(true)) | (Inf(false), _) => Some(Less),
            (Inf(true), _) | (_, Inf(false)) => Some(Greater),
            (Zero(s1), Zero(s2)) => s1.partial_cmp(s2),
            (Zero(_), Rational(q)) => Some(q.sign().reverse()),
            (Rational(q), Zero(_)) => Some(q.sign()),
            (Rational(q1), Rational(q2)) => q1.partial_cmp(q2),
        }
    }
}

impl Approx<Mpn> for Mpq {
    type Output = Self;
    fn approx(self, max_den: &Mpn) -> Self::Output {
        self.approximate(max_den)
    }
}

impl Approx<Mpn> for &Mpq {
    type Output = Mpq;
    fn approx(self, max_den: &Mpn) -> Self::Output {
        self.approximate(max_den)
    }
}

impl ApproxAssign<Mpn> for Mpq {
    fn approx_assign(&mut self, max_den: &Mpn) {
        self.approximate_assign(max_den);
    }
}

impl Approx<Mpn> for MpqExt {
    type Output = MpqExt;
    fn approx(self, max_den: &Mpn) -> Self::Output {
        use MpqExt::*;
        match self {
            NaN => NaN,
            Zero(s) => Zero(s),
            Inf(s) => Inf(s),
            Rational(q) => {
                let orig_sign = q.sign().is_gt();
                let new_value = q.approximate(max_den);
                match new_value {
                    Mpq::ZERO => Zero(orig_sign),
                    _ => Rational(new_value),
                }
            }
        }
    }
}

impl Approx<Mpn> for &MpqExt {
    type Output = MpqExt;
    fn approx(self, max_den: &Mpn) -> Self::Output {
        use MpqExt::*;
        match self {
            NaN => NaN,
            &Zero(s) => Zero(s),
            &Inf(s) => Inf(s),
            Rational(q) => {
                let orig_sign = q.sign().is_gt();
                let new_value = q.approximate(max_den);
                match new_value {
                    Mpq::ZERO => Zero(orig_sign),
                    _ => Rational(new_value),
                }
            }
        }
    }
}

impl ApproxAssign<Mpn> for MpqExt {
    fn approx_assign(&mut self, max_den: &Mpn) {
        use MpqExt::*;
        match self {
            rational @ Rational(_) => {
                if let Rational(q) = rational {
                    let orig_sign = q.sign().is_gt();
                    <Mpq as ApproximateAssign>::approximate_assign(q, max_den);
                    if q.sign().is_eq() {
                        *rational = Zero(orig_sign);
                    }
                }
            }
            _ => {}
        }
    }
}
