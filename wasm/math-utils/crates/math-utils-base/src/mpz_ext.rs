use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    iter::{Product, Sum},
    ops::*,
    str::FromStr,
};

use anyhow::anyhow;
use malachite::{
    Integer as Mpz, Natural as Mpn,
    base::{
        comparison::traits::{Max, Min},
        num::{
            arithmetic::traits::{NegAssign, Pow, PowAssign, Sign, UnsignedAbs},
            basic::traits::{
                Infinity, NaN, NegativeInfinity, NegativeOne, NegativeZero, One, Two, Zero,
            },
            conversion::traits::FromStringBase,
        },
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    MpnExt, impl_product, impl_sum,
    traits::{ExtendedNumber, PartialOrdStrict, SignStrict, Ten},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "SerdeMpzExt", into = "SerdeMpzExt")]
pub enum MpzExt {
    NaN,
    Zero(bool),
    Inf(bool),
    Integer(Mpz),
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct SerdeMpzExt(String);

impl From<MpzExt> for SerdeMpzExt {
    #[inline]
    fn from(value: MpzExt) -> Self {
        use MpzExt::*;
        match value {
            NaN => SerdeMpzExt("nan".into()),
            Zero(sign) => SerdeMpzExt(if sign { "0".into() } else { "-0".into() }),
            Inf(sign) => SerdeMpzExt(if sign { "inf".into() } else { "-inf".into() }),
            Integer(n) => SerdeMpzExt(format!("{n:#x}")),
        }
    }
}

impl TryFrom<SerdeMpzExt> for MpzExt {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(s: SerdeMpzExt) -> Result<MpzExt, anyhow::Error> {
        let src = s.0;
        use MpzExt::*;
        if src == "nan" {
            Ok(MpzExt::NaN)
        } else if src == "0" {
            Ok(MpzExt::ZERO)
        } else if src == "-0" {
            Ok(MpzExt::NEGATIVE_ZERO)
        } else if src == "inf" {
            Ok(MpzExt::INFINITY)
        } else if src == "-inf" {
            Ok(MpzExt::NEGATIVE_INFINITY)
        } else if src.starts_with('-') {
            if src.starts_with("-0x") {
                Ok(Integer(Mpz::from_sign_and_abs(
                    false,
                    Mpn::from_string_base(16, &src[3..])
                        .ok_or_else(|| anyhow!("Unrecognized digits in {}", src))?,
                )))
            } else {
                Err(anyhow!(format!(
                    "String '{}' starts with '-' but not with '-0x'",
                    src
                )))
            }
        } else if src.starts_with("0x") {
            Ok(Integer(Mpz::from(
                Mpn::from_string_base(16, &src[2..])
                    .ok_or_else(|| anyhow!("Unrecognized digits in {}", src))?,
            )))
        } else {
            Err(anyhow!(
                "String '{}' does not start with '0x' or '-0x'",
                src
            ))
        }
    }
}

impl Zero for MpzExt {
    const ZERO: Self = MpzExt::Zero(true);
}

impl NegativeZero for MpzExt {
    const NEGATIVE_ZERO: Self = MpzExt::Zero(false);
}

impl One for MpzExt {
    const ONE: Self = MpzExt::Integer(Mpz::ONE);
}

impl NegativeOne for MpzExt {
    const NEGATIVE_ONE: Self = MpzExt::Integer(Mpz::NEGATIVE_ONE);
}

impl Infinity for MpzExt {
    const INFINITY: Self = MpzExt::Inf(true);
}

impl NegativeInfinity for MpzExt {
    const NEGATIVE_INFINITY: Self = MpzExt::Inf(false);
}

impl NaN for MpzExt {
    const NAN: Self = MpzExt::NaN;
}

impl Two for MpzExt {
    const TWO: Self = MpzExt::Integer(Mpz::TWO);
}

impl Ten for MpzExt {
    const TEN: Self = MpzExt::Integer(Mpz::TEN);
}

impl Max for MpzExt {
    const MAX: Self = MpzExt::INFINITY;
}

impl Min for MpzExt {
    const MIN: Self = MpzExt::NEGATIVE_INFINITY;
}

impl ExtendedNumber for MpzExt {
    fn is_nan(&self) -> bool {
        matches!(self, MpzExt::NaN)
    }

    fn is_infinite(&self) -> bool {
        matches!(self, MpzExt::Inf(_))
    }
}

impl From<Mpz> for MpzExt {
    fn from(value: Mpz) -> Self {
        match value {
            Mpz::ZERO => MpzExt::ZERO,
            n => MpzExt::Integer(n),
        }
    }
}

impl TryInto<Mpz> for MpzExt {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Mpz, Self::Error> {
        match self {
            MpzExt::Integer(n) => Ok(n),
            MpzExt::Zero(_) => Ok(Mpz::ZERO),
            _ => Err(anyhow!("NaN and infinity cannot be converted")),
        }
    }
}

macro_rules! from_str_with_special {
    ($s:expr, $e:expr) => {
        if $s.eq_ignore_ascii_case("inf") | $s.eq_ignore_ascii_case("+inf") {
            MpzExt::INFINITY
        } else if $s.eq_ignore_ascii_case("-inf") {
            MpzExt::NEGATIVE_INFINITY
        } else if $s.eq_ignore_ascii_case("nan")
            | $s.eq_ignore_ascii_case("+nan")
            | $s.eq_ignore_ascii_case("-nan")
        {
            MpzExt::NAN
        } else {
            $e
        }
    };
}

impl FromStr for MpzExt {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use MpzExt::*;
        Ok(from_str_with_special!(s, {
            let n = Mpz::from_str(s).map_err(|_| anyhow!("parsing failed"))?;
            match n {
                Mpz::ZERO => Zero(!s.starts_with('-')),
                n => Integer(n),
            }
        }))
    }
}

macro_rules! impl_mpz_ext_from_int {
    ($($t:ty),+$(,)?) => {
        $(
            impl From<$t> for MpzExt {
                fn from(value: $t) -> MpzExt {
                    use MpzExt::*;
                    match value {
                        0 => MpzExt::ZERO,
                        n => Integer(n.into()),
                    }
                }
            }
        )*
    };
}

impl_mpz_ext_from_int!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

impl FromStringBase for MpzExt {
    fn from_string_base(base: u8, s: &str) -> Option<Self> {
        use MpzExt::*;
        Some(from_str_with_special!(s, {
            let n = Mpz::from_string_base(base, s)?;
            match n {
                Mpz::ZERO => Zero(!s.starts_with('-')),
                n => Integer(n),
            }
        }))
    }
}

impl Neg for MpzExt {
    type Output = Self;

    fn neg(self) -> Self::Output {
        use MpzExt::*;
        match self {
            NaN => NaN,
            Zero(s) => Zero(!s),
            Inf(s) => Inf(!s),
            Integer(n) => Integer(-n),
        }
    }
}

impl Neg for &MpzExt {
    type Output = MpzExt;

    fn neg(self) -> Self::Output {
        use MpzExt::*;
        match self {
            NaN => NaN,
            &Zero(s) => Zero(!s),
            &Inf(s) => Inf(!s),
            Integer(n) => Integer(-n),
        }
    }
}

impl NegAssign for MpzExt {
    fn neg_assign(&mut self) {
        use MpzExt::*;
        match self {
            Zero(s) | Inf(s) => *s = !*s,
            Integer(n) => n.neg_assign(),
            NaN => {}
        }
    }
}

impl Add for MpzExt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use MpzExt::*;
        match (self, rhs) {
            (Inf(s1), Inf(s2)) if s1 != s2 => NaN,
            (a @ NaN, _)
            | (_, a @ NaN)
            | (a @ Zero(true), Zero(_))
            | (Zero(_), a @ Zero(true))
            | (a @ Zero(false), Zero(false))
            | (a, Zero(_))
            | (Zero(_), a)
            | (a @ Inf(_), _)
            | (_, a @ Inf(_)) => a,
            (Integer(m), Integer(n)) => (m + n).into(),
        }
    }
}

impl Add<&Self> for MpzExt {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        use MpzExt::*;
        match (self, rhs) {
            (Inf(s1), &Inf(s2)) if s1 != s2 => NaN,
            (a @ NaN, _) => a,
            (_, NaN) => NaN,
            (a @ Zero(true), Zero(_)) => a,
            (Zero(_), Zero(true)) => Zero(true),
            (a @ Zero(false), Zero(false)) | (a, Zero(_)) => a,
            (Zero(_), a) => a.clone(),
            (a @ Inf(_), _) => a,
            (_, a @ Inf(_)) => a.clone(),
            (Integer(m), Integer(n)) => (m + n).into(),
        }
    }
}

impl Add<MpzExt> for &MpzExt {
    type Output = MpzExt;

    fn add(self, rhs: MpzExt) -> Self::Output {
        use MpzExt::*;
        match (self, rhs) {
            (&Inf(s1), Inf(s2)) if s1 != s2 => NaN,
            (NaN, _) => MpzExt::NAN,
            (_, a @ NaN) => a,
            (Zero(true), Zero(_)) => MpzExt::ZERO,
            (Zero(_), a @ Zero(true)) | (Zero(false), a @ Zero(false)) => a,
            (a, Zero(_)) => a.clone(),
            (Zero(_), a) => a,
            (a @ Inf(_), _) => a.clone(),
            (_, a @ Inf(_)) => a,
            (Integer(m), Integer(n)) => (m + n).into(),
        }
    }
}

impl Add for &MpzExt {
    type Output = MpzExt;

    fn add(self, rhs: Self) -> Self::Output {
        use MpzExt::*;
        match (self, rhs) {
            (Inf(s1), Inf(s2)) if s1 != s2 => NaN,
            (a @ NaN, _)
            | (_, a @ NaN)
            | (a @ Zero(true), Zero(_))
            | (Zero(_), a @ Zero(true))
            | (a @ Zero(false), Zero(false))
            | (a, Zero(_))
            | (Zero(_), a)
            | (a @ Inf(_), _)
            | (_, a @ Inf(_)) => a.clone(),
            (Integer(m), Integer(n)) => (m + n).into(),
        }
    }
}

impl AddAssign for MpzExt {
    fn add_assign(&mut self, rhs: Self) {
        use MpzExt::*;
        match (self, rhs) {
            (a @ Inf(_), b @ Inf(_)) => {
                if let &mut Inf(s1) = a {
                    if let Inf(s2) = b {
                        if s1 != s2 {
                            *a = Self::NAN;
                        }
                    } else {
                        unreachable!();
                    }
                } else {
                    unreachable!();
                }
            }
            (Zero(s1), Zero(s2)) => {
                *s1 |= s2;
            }
            (NaN, _) | (_, Zero(_)) => {}
            (a, NaN) => *a = Self::NAN,
            (Inf(_), _) => {}
            (a, b @ Inf(_)) | (a @ Zero(_), b) => *a = b,
            (a @ Integer(_), b @ Integer(_)) => {
                if let Integer(m) = a {
                    if let Integer(n) = b {
                        *m += n;
                        if *m == Mpz::ZERO {
                            *a = Zero(true);
                        }
                    } else {
                        unreachable!();
                    }
                } else {
                    unreachable!();
                }
            }
        }
    }
}

impl AddAssign<&Self> for MpzExt {
    fn add_assign(&mut self, rhs: &Self) {
        use MpzExt::*;
        match (self, rhs) {
            (a @ Inf(_), b @ Inf(_)) => {
                if let &mut Inf(s1) = a {
                    if let &Inf(s2) = b {
                        if s1 != s2 {
                            *a = Self::NAN;
                        }
                    } else {
                        unreachable!();
                    }
                } else {
                    unreachable!();
                }
            }
            (Zero(s1), Zero(s2)) => {
                *s1 |= s2;
            }
            (NaN, _) | (_, Zero(_)) => {}
            (a, NaN) => *a = Self::NAN,
            (Inf(_), _) => {}
            (a, b @ Inf(_)) | (a @ Zero(_), b) => *a = b.clone(),
            (a @ Integer(_), b @ Integer(_)) => {
                if let Integer(m) = a {
                    if let Integer(n) = b {
                        *m += n;
                        if *m == Mpz::ZERO {
                            *a = Self::ZERO;
                        }
                    } else {
                        unreachable!();
                    }
                } else {
                    unreachable!();
                }
            }
        }
    }
}

impl_sum!(MpzExt);

impl Mul for MpzExt {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        use MpzExt::*;
        match (self, rhs) {
            (Inf(_), Zero(_)) | (Zero(_), Inf(_)) | (NaN, _) | (_, NaN) => Self::NAN,
            (Inf(s1), Inf(s2)) => Inf(s1 == s2),
            (Zero(s1), Zero(s2)) => Zero(s1 == s2),
            (Zero(s1), Integer(m)) | (Integer(m), Zero(s1)) => {
                let s2 = m.sign().is_gt();
                Zero(s1 == s2)
            }
            (Inf(s1), Integer(m)) | (Integer(m), Inf(s1)) => {
                let s2 = m.sign().is_gt();
                Inf(s1 == s2)
            }
            (Integer(m), Integer(n)) => Integer(m * n),
        }
    }
}

impl MulAssign for MpzExt {
    fn mul_assign(&mut self, rhs: Self) {
        use MpzExt::*;
        match (self, rhs) {
            (NaN, _) => {}
            (a @ Inf(_), Zero(_)) | (a @ Zero(_), Inf(_)) | (a, NaN) => *a = Self::NAN,
            (Zero(s1), Zero(s2)) | (Inf(s1), Inf(s2)) => *s1 = *s1 == s2,
            (Zero(s1), Integer(m)) | (Inf(s1), Integer(m)) => {
                let s2 = m.sign().is_gt();
                *s1 = *s1 == s2;
            }
            (a @ Integer(_), Zero(s1)) | (a @ Integer(_), Inf(s1)) => {
                if let Integer(m) = a {
                    let s2 = m.sign().is_gt();
                    *a = Zero(s1 == s2);
                } else {
                    unreachable!();
                }
            }
            (Integer(m), Integer(n)) => {
                *m *= n;
            }
        }
    }
}

impl MulAssign<&Self> for MpzExt {
    fn mul_assign(&mut self, rhs: &Self) {
        use MpzExt::*;
        match (self, rhs) {
            (NaN, _) => {}
            (a @ Inf(_), Zero(_)) | (a @ Zero(_), Inf(_)) | (a, NaN) => *a = Self::NAN,
            (Zero(s1), &Zero(s2)) | (Inf(s1), &Inf(s2)) => *s1 = *s1 == s2,
            (Zero(s1), Integer(m)) | (Inf(s1), Integer(m)) => {
                let s2 = m.sign().is_gt();
                *s1 = *s1 == s2;
            }
            (a @ Integer(_), &Zero(s1)) | (a @ Integer(_), &Inf(s1)) => {
                if let Integer(m) = a {
                    let s2 = m.sign().is_gt();
                    *a = Zero(s1 == s2);
                } else {
                    unreachable!();
                }
            }
            (Integer(m), Integer(n)) => {
                *m *= n;
            }
        }
    }
}

impl_product!(MpzExt);

impl Sub for MpzExt {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        use MpzExt::*;
        match (self, rhs) {
            (Inf(s1), Inf(s2)) if s1 == s2 => NaN,
            (a @ NaN, _) | (_, a @ NaN) | (a @ Zero(true), Zero(_)) => a,
            (Zero(_), Zero(false)) => MpzExt::ZERO,
            (a @ Zero(false), Zero(true)) | (a, Zero(_)) => a,
            (Zero(_), Integer(m)) => Integer(-m),
            (a @ Inf(_), _) => a,
            (_, Inf(s)) => Inf(!s),
            (Integer(m), Integer(n)) => (m - n).into(),
        }
    }
}

// [TODO] impl Sub<&Self> for MpzExt
// [TODO] impl Sub<MpzExt> for &MpzExt
// [TODO] impl Sub for &MpzExt
// [TODO] impl SubAssign for MpzExt
// [TODO] impl SubAssign<&Self> for MpzExt

impl Div for MpzExt {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        use MpzExt::*;
        match (self, rhs) {
            (a @ NaN, _) => a,
            (_, NaN) | (Zero(_), Zero(_)) | (Inf(_), Inf(_)) => Self::NAN,
            (Zero(s1), Integer(n)) | (Integer(n), Inf(s1)) => {
                let s2 = n.sign().is_gt();
                Zero(s1 == s2)
            }
            (Integer(n), Zero(s1)) | (Inf(s1), Integer(n)) => {
                let s2 = n.sign().is_gt();
                Inf(s1 == s2)
            }
            (Zero(s1), Inf(s2)) => Zero(s1 == s2),
            (Inf(s1), Zero(s2)) => Inf(s1 == s2),
            (Integer(m), Integer(n)) => {
                let result_sign = m.sign() == n.sign();
                let result = m / n;
                match result {
                    Mpz::ZERO => Zero(result_sign),
                    result => Integer(result),
                }
            }
        }
    }
}

// [TODO] impl Div<&Self> for MpzExt
// [TODO] impl Div<MpzExt> for &MpzExt
// [TODO] impl Div for &MpzExt
// [TODO] impl DivAssign for MpzExt
// [TODO] impl DivAssign<&Self> for MpzExt

impl PartialEq for MpzExt {
    fn eq(&self, other: &Self) -> bool {
        use MpzExt::*;
        match (self, other) {
            (Zero(_), Zero(_)) => true,
            (Inf(s1), Inf(s2)) => s1 == s2,
            (Integer(m), Integer(n)) => m == n,
            _ => false,
        }
    }
}

impl Sign for MpzExt {
    fn sign(&self) -> Ordering {
        use MpzExt::*;
        use Ordering::*;
        match self {
            NaN | Zero(_) => Equal,
            Inf(true) => Greater,
            Inf(false) => Less,
            Integer(n) => n.sign(),
        }
    }
}

impl SignStrict for MpzExt {
    fn sign_strict(&self) -> Ordering {
        use MpzExt::*;
        use Ordering::*;
        match self {
            NaN => Equal,
            Inf(true) | Zero(true) => Greater,
            Inf(false) | Zero(false) => Less,
            Integer(n) => n.sign(),
        }
    }
}

impl PartialOrd for MpzExt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use MpzExt::*;
        use Ordering::*;
        match (self, other) {
            (NaN, _) | (_, NaN) => None,
            (Zero(_), Zero(_)) => Some(Equal),
            (Inf(s1), Inf(s2)) if s1 == s2 => Some(Equal),
            (Inf(true), _) | (_, Inf(false)) => Some(Greater),
            (_, Inf(true)) | (Inf(false), _) => Some(Less),
            (a @ Integer(_), Zero(_)) => a.sign().into(),
            (Zero(_), a @ Integer(_)) => a.sign().reverse().into(),
            (a @ Integer(_), b @ Integer(_)) => a.partial_cmp(b),
        }
    }
}

impl PartialOrdStrict for MpzExt {
    fn partial_cmp_strict(&self, other: &Self) -> Option<Ordering> {
        use MpzExt::*;
        use Ordering::*;
        match (self, other) {
            (NaN, _) | (_, NaN) => None,
            (Zero(s1), Zero(s2)) if s1 == s2 => Some(Equal),
            (Zero(true), Zero(_)) => Some(Greater),
            (Zero(_), Zero(_)) => Some(Less),
            (Inf(s1), Inf(s2)) if s1 == s2 => Some(Equal),
            (Inf(true), _) | (_, Inf(false)) => Some(Greater),
            (_, Inf(true)) | (Inf(false), _) => Some(Less),
            (a @ Integer(_), Zero(_)) => a.sign().into(),
            (Zero(_), a @ Integer(_)) => a.sign().reverse().into(),
            (a @ Integer(_), b @ Integer(_)) => a.partial_cmp(b),
        }
    }
}

impl UnsignedAbs for MpzExt {
    type Output = MpnExt;
    fn unsigned_abs(self) -> Self::Output {
        use MpzExt::*;
        match self {
            NaN => MpnExt::NAN,
            Zero(_) => MpnExt::ZERO,
            Inf(_) => MpnExt::INFINITY,
            Integer(n) => MpnExt::Integer(n.unsigned_abs()),
        }
    }
}

impl Display for MpzExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MpzExt::*;
        match self {
            NaN => write!(f, "nan"),
            Zero(true) => write!(f, "0"),
            Zero(false) => write!(f, "-0"),
            Inf(true) => write!(f, "inf"),
            Inf(false) => write!(f, "-inf"),
            Integer(n) => Display::fmt(n, f),
        }
    }
}

impl Debug for MpzExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl PowAssign<u64> for MpzExt {
    fn pow_assign(&mut self, exp: u64) {
        use MpzExt::*;
        match self {
            NaN => {}
            Zero(true) | Inf(true) => {
                if exp == 0 {
                    *self = MpzExt::ONE;
                }
            }
            Zero(s @ false) | Inf(s @ false) => {
                if exp == 0 {
                    *self = MpzExt::ONE;
                } else {
                    *s = exp % 2 == 0;
                }
            }
            Integer(n) => n.pow_assign(exp),
        }
    }
}

impl Pow<u64> for MpzExt {
    type Output = Self;

    fn pow(mut self, exp: u64) -> Self::Output {
        self.pow_assign(exp);
        self
    }
}

impl Pow<u64> for &MpzExt {
    type Output = MpzExt;

    fn pow(self, exp: u64) -> Self::Output {
        let mut result = self.clone();
        result.pow_assign(exp);
        result
    }
}
