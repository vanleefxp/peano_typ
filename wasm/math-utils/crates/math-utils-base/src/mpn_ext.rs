use std::{
    cmp::{Ordering, PartialOrd},
    fmt::{Debug, Display},
    iter::{Product, Sum},
    ops::*,
    str::FromStr,
};

use anyhow::{anyhow, bail};
use malachite::{
    Natural as Mpn,
    base::{
        comparison::traits::{Max, Min},
        num::{
            arithmetic::traits::{CheckedSub, Sign},
            basic::traits::{Infinity, NaN, One, Two, Zero},
            conversion::traits::FromStringBase,
        },
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    impl_product, impl_sum,
    traits::{ExtendedNumber, SignStrict, Ten},
};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "SerdeMpnExt", into = "SerdeMpnExt")]
pub enum MpnExt {
    NaN,
    Inf,
    Zero,
    Integer(Mpn),
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct SerdeMpnExt(String);

impl From<MpnExt> for SerdeMpnExt {
    fn from(value: MpnExt) -> Self {
        SerdeMpnExt(match value {
            MpnExt::NaN => "nan".into(),
            MpnExt::Inf => "inf".into(),
            MpnExt::Zero => "0".into(),
            MpnExt::Integer(n) => format!("{n:#x}"),
        })
    }
}

impl TryFrom<SerdeMpnExt> for MpnExt {
    type Error = anyhow::Error;

    fn try_from(value: SerdeMpnExt) -> Result<MpnExt, Self::Error> {
        use MpnExt::*;
        let src = value.0.as_str();
        match src {
            "nan" => Ok(MpnExt::NAN),
            "inf" => Ok(MpnExt::INFINITY),
            "0" => Ok(MpnExt::ZERO),
            src => {
                if src.starts_with("0x") {
                    Ok(Integer(Mpn::from_string_base(16, &src[2..]).ok_or_else(
                        || anyhow!("Unrecognized digits in {}", src),
                    )?))
                } else {
                    bail!("String '{}' does not start with '0x'", src);
                }
            }
        }
    }
}

impl Zero for MpnExt {
    const ZERO: Self = Self::Zero;
}

impl Infinity for MpnExt {
    const INFINITY: Self = Self::Inf;
}

impl NaN for MpnExt {
    const NAN: Self = Self::NaN;
}

impl Min for MpnExt {
    const MIN: Self = Self::Zero;
}

impl Max for MpnExt {
    const MAX: Self = Self::Inf;
}

impl One for MpnExt {
    const ONE: Self = Self::Integer(Mpn::ONE);
}

impl Two for MpnExt {
    const TWO: Self = Self::Integer(Mpn::TWO);
}

impl Ten for MpnExt {
    const TEN: Self = Self::Integer(Mpn::TEN);
}

impl ExtendedNumber for MpnExt {
    fn is_nan(&self) -> bool {
        matches!(self, MpnExt::NaN)
    }

    fn is_infinite(&self) -> bool {
        matches!(self, MpnExt::Inf)
    }
}

impl From<Mpn> for MpnExt {
    fn from(value: Mpn) -> Self {
        use MpnExt::*;
        match value {
            Mpn::ZERO => Zero,
            value => Integer(value),
        }
    }
}

impl From<&Mpn> for MpnExt {
    fn from(value: &Mpn) -> Self {
        use MpnExt::*;
        match value {
            &Mpn::ZERO => Zero,
            value => Integer(value.clone()),
        }
    }
}

impl TryInto<Mpn> for MpnExt {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Mpn, Self::Error> {
        use MpnExt::*;
        match self {
            Zero => Ok(Mpn::ZERO),
            Integer(n) => Ok(n),
            _ => bail!("Infinity and NaN cannot be converted"),
        }
    }
}

impl FromStr for MpnExt {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use MpnExt::*;
        Ok(if s.eq_ignore_ascii_case("inf") {
            Inf
        } else if s.eq_ignore_ascii_case("nan") {
            NaN
        } else {
            let n = Mpn::from_str(s).map_err(|_| anyhow!("parsing failed"))?;
            match n {
                Mpn::ZERO => Zero,
                n => Integer(n),
            }
        })
    }
}

macro_rules! impl_mpz_ext_from_int {
    ($($t:ty),+$(,)?) => {
        $(
            impl From<$t> for MpnExt {
                fn from(value: $t) -> MpnExt {
                    use MpnExt::*;
                    match value {
                        0 => MpnExt::ZERO,
                        n => Integer(n.into()),
                    }
                }
            }
        )*
    };
}

impl_mpz_ext_from_int!(u8, u16, u32, u64, u128, usize);

impl FromStringBase for MpnExt {
    fn from_string_base(base: u8, s: &str) -> Option<Self> {
        use MpnExt::*;
        if s.eq_ignore_ascii_case("inf") {
            Some(MpnExt::INFINITY)
        } else if s.eq_ignore_ascii_case("nan") {
            Some(MpnExt::NAN)
        } else {
            if let Some(n) = Mpn::from_string_base(base, s) {
                match n {
                    Mpn::ZERO => Some(MpnExt::ZERO),
                    n => Some(Integer(n)),
                }
            } else {
                None
            }
        }
    }
}

impl Add for MpnExt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use MpnExt::*;
        match (self, rhs) {
            (a @ (NaN | Inf), _) | (a, Zero) | (_, a @ (NaN | Inf)) | (Zero, a) => a,
            (Integer(m), Integer(n)) => Integer(m + n),
        }
    }
}

impl Add<&Self> for MpnExt {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        use MpnExt::*;
        match (self, rhs) {
            (a @ (NaN | Inf), _) | (a, Zero) => a,
            (_, a @ (NaN | Inf)) | (Zero, a) => a.clone(),
            (Integer(m), Integer(n)) => Integer(m + n),
        }
    }
}

impl Add<MpnExt> for &MpnExt {
    type Output = MpnExt;

    fn add(self, rhs: MpnExt) -> Self::Output {
        use MpnExt::*;
        match (self, rhs) {
            (a @ (NaN | Inf), _) | (a, Zero) => a.clone(),
            (_, a @ (NaN | Inf)) | (Zero, a) => a,
            (Integer(m), Integer(n)) => Integer(m + n),
        }
    }
}

impl Add<Self> for &MpnExt {
    type Output = MpnExt;

    fn add(self, rhs: Self) -> Self::Output {
        use MpnExt::*;
        match (self, rhs) {
            (a @ (NaN | Inf), _) | (a, Zero) | (_, a @ (NaN | Inf)) | (Zero, a) => a.clone(),
            (Integer(m), Integer(n)) => Integer(m + n),
        }
    }
}

impl AddAssign for MpnExt {
    fn add_assign(&mut self, rhs: Self) {
        use MpnExt::*;
        match (self, rhs) {
            (NaN | Inf, _) | (_, Zero) => {}
            (a, b @ (NaN | Inf)) | (a @ Zero, b) => *a = b,
            (Integer(m), Integer(n)) => *m += n,
        }
    }
}

impl AddAssign<&Self> for MpnExt {
    fn add_assign(&mut self, rhs: &Self) {
        use MpnExt::*;
        match (self, rhs) {
            (NaN | Inf, _) | (_, Zero) => {}
            (a, b @ (NaN | Inf)) | (a @ Zero, b) => *a = b.clone(),
            (Integer(m), Integer(n)) => *m += n,
        }
    }
}

impl_sum!(MpnExt);

impl Mul for MpnExt {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        use MpnExt::*;
        match (self, rhs) {
            (Zero, Inf) | (Inf, Zero) => NaN,
            (a @ (NaN | Zero | Inf), _) | (_, a @ (NaN | Zero | Inf)) => a,
            (Integer(m), Integer(n)) => Integer(m * n),
        }
    }
}

impl Mul<&Self> for MpnExt {
    type Output = Self;

    fn mul(self, rhs: &Self) -> Self::Output {
        use MpnExt::*;
        match (self, rhs) {
            (Zero, Inf) | (Inf, Zero) => NaN,
            (a @ (NaN | Zero | Inf), _) => a,
            (_, a @ (NaN | Zero | Inf)) => a.clone(),
            (Integer(m), Integer(n)) => Integer(m * n),
        }
    }
}

impl Mul<MpnExt> for &MpnExt {
    type Output = MpnExt;

    fn mul(self, rhs: MpnExt) -> Self::Output {
        use MpnExt::*;
        match (self, rhs) {
            (Zero, Inf) | (Inf, Zero) => NaN,
            (a @ (NaN | Zero | Inf), _) => a.clone(),
            (_, a @ (NaN | Zero | Inf)) => a,
            (Integer(m), Integer(n)) => Integer(m * n),
        }
    }
}

impl Mul for &MpnExt {
    type Output = MpnExt;

    fn mul(self, rhs: Self) -> Self::Output {
        use MpnExt::*;
        match (self, rhs) {
            (Zero, Inf) | (Inf, Zero) => NaN,
            (a @ (NaN | Zero | Inf), _) | (_, a @ (NaN | Zero | Inf)) => a.clone(),
            (Integer(m), Integer(n)) => Integer(m * n),
        }
    }
}

impl MulAssign for MpnExt {
    fn mul_assign(&mut self, rhs: Self) {
        use MpnExt::*;
        match (self, rhs) {
            (a @ Zero, Inf) | (a @ Inf, Zero) => *a = NaN,
            (NaN | Zero | Inf, _) => {}
            (a, b @ (NaN | Zero | Inf)) => *a = b,
            (Integer(m), Integer(n)) => *m *= n,
        }
    }
}

impl MulAssign<&Self> for MpnExt {
    fn mul_assign(&mut self, rhs: &Self) {
        use MpnExt::*;
        match (self, rhs) {
            (a @ Zero, Inf) | (a @ Inf, Zero) => *a = NaN,
            (NaN | Zero | Inf, _) => {}
            (a, b @ (NaN | Zero | Inf)) => *a = b.clone(),
            (Integer(m), Integer(n)) => *m *= n,
        }
    }
}

impl_product!(MpnExt);

impl CheckedSub for MpnExt {
    type Output = Self;

    fn checked_sub(self, other: Self) -> Option<Self::Output> {
        use MpnExt::*;
        match (self, other) {
            (Inf, Inf) => Some(NaN),
            (a @ NaN, _) | (_, a @ NaN) | (a @ Inf, _) | (a, Zero) => Some(a),
            (Zero, _) | (_, Inf) => None,
            (Integer(m), Integer(n)) => m.checked_sub(n).map(Integer),
        }
    }
}

impl CheckedSub<&Self> for MpnExt {
    type Output = Self;

    fn checked_sub(self, other: &Self) -> Option<Self::Output> {
        use MpnExt::*;
        match (self, other) {
            (Inf, Inf) | (_, NaN) => Some(NaN),
            (a @ NaN, _) | (a @ Inf, _) | (a, Zero) => Some(a),
            (Zero, _) | (_, Inf) => None,
            (Integer(m), Integer(n)) => m.checked_sub(n).map(Integer),
        }
    }
}

impl CheckedSub<MpnExt> for &MpnExt {
    type Output = MpnExt;

    fn checked_sub(self, other: MpnExt) -> Option<Self::Output> {
        use MpnExt::*;
        match (self, other) {
            (Inf, Inf) => Some(NaN),
            (_, a @ NaN) => Some(a),
            (a @ NaN, _) | (a @ Inf, _) | (a, Zero) => Some(a.clone()),
            (Zero, _) | (_, Inf) => None,
            (Integer(m), Integer(n)) => m.checked_sub(n).map(Integer),
        }
    }
}

impl CheckedSub for &MpnExt {
    type Output = MpnExt;

    fn checked_sub(self, other: Self) -> Option<Self::Output> {
        use MpnExt::*;
        match (self, other) {
            (Inf, Inf) => Some(NaN),
            (a @ NaN, _) | (_, a @ NaN) | (a @ Inf, _) | (a, Zero) => Some(a.clone()),
            (Zero, _) | (_, Inf) => None,
            (Integer(m), Integer(n)) => m.checked_sub(n).map(Integer),
        }
    }
}

macro_rules! impl_sub_for_mpn_ext {
    ($t1:ty, $t2:ty) => {
        impl Sub<$t2> for $t1 {
            type Output = MpnExt;

            fn sub(self, rhs: $t2) -> Self::Output {
                self.checked_sub(rhs)
                    .expect("Cannot subtract a Natural from a smaller Natural")
            }
        }
    };
}

impl_sub_for_mpn_ext!(MpnExt, MpnExt);
impl_sub_for_mpn_ext!(MpnExt, &MpnExt);
impl_sub_for_mpn_ext!(&MpnExt, MpnExt);
impl_sub_for_mpn_ext!(&MpnExt, &MpnExt);

impl PartialOrd for MpnExt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use MpnExt::*;
        use Ordering::*;
        match (self, other) {
            (NaN, _) | (_, NaN) => None,
            (Inf, Inf) | (Zero, Zero) => Some(Equal),
            (Inf, _) | (_, Zero) => Some(Greater),
            (_, Inf) | (Zero, _) => Some(Less),
            (Integer(m), Integer(n)) => m.partial_cmp(n),
        }
    }
}

impl Sign for MpnExt {
    fn sign(&self) -> Ordering {
        use MpnExt::*;
        use Ordering::*;
        match self {
            NaN | Zero => Equal,
            _ => Greater,
        }
    }
}

impl SignStrict for MpnExt {
    fn sign_strict(&self) -> Ordering {
        use MpnExt::*;
        use Ordering::*;
        match self {
            NaN => Equal,
            _ => Greater,
        }
    }
}

impl Display for MpnExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MpnExt::NaN => write!(f, "nan"),
            MpnExt::Inf => write!(f, "inf"),
            MpnExt::Zero => Display::fmt(&Mpn::ZERO, f),
            MpnExt::Integer(n) => Display::fmt(n, f),
        }
    }
}

impl Debug for MpnExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
