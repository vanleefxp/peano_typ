use std::ops::Deref;
use std::{num::ParseIntError, str::FromStr};

use fraction::{ConstOne, Ratio};
use fraction::{GenericFraction, generic::GenericInteger};
use malachite::base::num::arithmetic::traits::Pow;
use malachite::base::num::{
    arithmetic::traits::Sign,
    basic::traits::{One, Zero},
};
use num::integer::Integer;
use serde::{Deserialize, Serialize};

use math_utils_base::{parsing::*, traits::*};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct FracData<T>
where
    T: Integer + Clone + Copy,
{
    pub sign: bool,
    pub num: T,
    pub den: T,
}

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
        + From<u8>,
{
    type Err = anyhow::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(ParseFractionResult::from_str(src)?.into())
    }
}

// Extending malachite::Rational with infinity and NaN support

// impl Ceiling for MpqExt {
//     type Output = Self;

//     fn ceiling(self) -> Self::Output {
//         use MpqExt::*;
//         match self {
//             Zero(_) | Inf(_) | NaN => self,
//             Rational(q) => Rational(q.ceiling()),
//         }
//     }
// }
