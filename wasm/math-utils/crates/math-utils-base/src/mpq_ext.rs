use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    iter::{Product, Sum},
    ops::*,
    str::FromStr,
};

use anyhow::anyhow;
use malachite::{
    Integer as Mpz, Natural as Mpn, Rational as Mpq,
    base::{
        comparison::traits::{Max, Min},
        num::{arithmetic::traits::*, basic::traits::*},
    },
};
use serde::{Deserialize, Serialize};

use crate::{MpnExt, MpzExt, impl_product, impl_sum, parsing::*, traits::*};

#[derive(Clone, Serialize, Deserialize, Hash)]
pub enum MpqExt {
    Zero(bool),
    Inf(bool),
    NaN,
    Rational(Mpq),
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
        use MpqExt::*;
        match (n, d) {
            (Mpz::ZERO, Mpz::ZERO) => Self::NAN,
            (Mpz::ZERO, d) => Zero(d.sign().is_gt()),
            (n, Mpz::ZERO) => Inf(n.sign().is_gt()),
            (n, d) => Rational(Mpq::from_integers(n, d)),
        }
    }

    pub fn from_extended_integers(n: MpzExt, d: MpzExt) -> Self {
        use MpzExt::*;
        match (n, d) {
            (Zero(_), Zero(_)) | (NaN, _) | (_, NaN) | (Inf(_), Inf(_)) => Self::NAN,
            (Zero(s1), Inf(s2)) => Self::Zero(s1 == s2),
            (Inf(s1), Zero(s2)) => Self::Inf(s1 == s2),
            (Zero(s1), Integer(n)) | (Integer(n), Inf(s1)) => {
                let s2 = n.sign().is_gt();
                Self::Zero(s1 == s2)
            }
            (Integer(m), Zero(s2)) | (Inf(s2), Integer(m)) => {
                let s1 = m.sign().is_gt();
                Self::Inf(s1 == s2)
            }
            (Integer(m), Integer(n)) => Self::from_integers(m, n),
        }
    }

    pub fn from_integers_ref(n: &Mpz, d: &Mpz) -> Self {
        use MpqExt::*;
        match (n, d) {
            (&Mpz::ZERO, &Mpz::ZERO) => NaN,
            (&Mpz::ZERO, d) => Zero(d.sign().is_gt()),
            (n, &Mpz::ZERO) => Inf(n.sign().is_gt()),
            (n, d) => Rational(Mpq::from_integers_ref(n, d)),
        }
    }

    pub fn from_extended_integers_ref(n: &MpzExt, d: &MpzExt) -> Self {
        use MpzExt::*;
        match (n, d) {
            (Zero(_), Zero(_)) | (NaN, _) | (_, NaN) | (Inf(_), Inf(_)) => Self::NAN,
            (Zero(s1), Inf(s2)) => Self::Zero(s1 == s2),
            (Inf(s1), Zero(s2)) => Self::Inf(s1 == s2),
            (&Zero(s1), Integer(n)) | (Integer(n), &Inf(s1)) => {
                let s2 = n.sign().is_gt();
                Self::Zero(s1 == s2)
            }
            (Integer(m), &Zero(s2)) | (&Inf(s2), Integer(m)) => {
                let s1 = m.sign().is_gt();
                Self::Inf(s1 == s2)
            }
            (Integer(m), Integer(n)) => Self::from_integers_ref(m, n),
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
        Ok(ParseFractionResult::from_str(src)?.into())
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

impl From<MpzExt> for MpqExt {
    fn from(value: MpzExt) -> Self {
        use MpzExt::*;
        match value {
            NaN => Self::NAN,
            Zero(s) => Self::Zero(s),
            Inf(s) => Self::Inf(s),
            Integer(n) => Self::Rational(n.into()),
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

impl From<MpnExt> for MpqExt {
    fn from(value: MpnExt) -> Self {
        use MpnExt::*;
        match value {
            NaN => Self::NAN,
            Zero => Self::ZERO,
            Inf => Self::INFINITY,
            Integer(n) => Self::Rational(n.into()),
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

impl Ten for MpqExt {
    const TEN: Self = Self::Rational(Mpq::TEN);
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

impl Max for MpqExt {
    const MAX: Self = Self::INFINITY;
}

impl Min for MpqExt {
    const MIN: Self = Self::NEGATIVE_INFINITY;
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

impl_sum!(MpqExt);

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

impl_product!(MpqExt);

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

impl PowAssign<u64> for MpqExt {
    fn pow_assign(&mut self, exp: u64) {
        use MpqExt::*;
        match self {
            NaN => {}
            Zero(true) | Inf(true) => {
                if exp == 0 {
                    *self = MpqExt::ONE;
                }
            }
            Zero(s @ false) | Inf(s @ false) => {
                if exp == 0 {
                    *self = MpqExt::ONE;
                } else {
                    *s = exp % 2 == 0;
                }
            }
            Rational(q) => {
                q.pow_assign(exp);
            }
        }
    }
}

impl PowAssign<i64> for MpqExt {
    fn pow_assign(&mut self, exp: i64) {
        use MpqExt::*;
        if matches!(self, NaN) || exp == 1 {
            return;
        }
        if exp == 0 {
            *self = MpqExt::ONE;
        }
        match self {
            Zero(true) if exp < 0 => *self = MpqExt::INFINITY,
            Zero(s @ false) if exp > 0 => *s = exp % 2 == 0,
            Zero(false) => *self = Inf(exp % 2 == 0),
            Inf(true) if exp < 0 => *self = MpqExt::ZERO,
            Inf(s @ false) if exp > 0 => *s = exp % 2 == 0,
            Inf(false) => *self = Zero(exp % 2 == 0),
            _ => {}
        }
    }
}

macro_rules! impl_pow_for_mpq_ext {
    ($($t:ty),+$(,)?) => {
        $(
            impl Pow<$t> for MpqExt {
                type Output = Self;
                fn pow(mut self, exp: $t) -> Self {
                    self.pow_assign(exp);
                    self
                }
            }

            impl Pow<$t> for &MpqExt {
                type Output = MpqExt;

                fn pow(self, exp: $t) -> Self::Output {
                    let mut result = self.clone();
                    result.pow_assign(exp);
                    result
                }
            }
        )*
    };
}

impl_pow_for_mpq_ext!(u64, i64);

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

impl PartialOrdStrict for MpqExt {
    fn partial_cmp_strict(&self, other: &Self) -> Option<Ordering> {
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
                let new_value = q.approx(max_den);
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
                let new_value = q.approx(max_den);
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
                    q.approx_assign(max_den);
                    if q.sign().is_eq() {
                        *rational = Zero(orig_sign);
                    }
                }
            }
            _ => {}
        }
    }
}
