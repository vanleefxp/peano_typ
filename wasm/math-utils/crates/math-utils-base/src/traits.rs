use std::cmp::Ordering;

use malachite::{
    Integer as Mpz, Natural as Mpn, Rational as Mpq,
    base::num::{arithmetic::traits::Sign, basic::traits::Zero},
    rational::arithmetic::traits::{Approximate, ApproximateAssign},
};

// replacing `malachite`'s  Approximate and ApproximateAssign traits to allow for customized output types.

pub trait Approx<N> {
    type Output;
    fn approx(self, max_den: &N) -> Self::Output;
}

pub trait ApproxAssign<N> {
    fn approx_assign(&mut self, max_den: &N);
}

pub trait Ten {
    const TEN: Self;
}

pub trait SignStrict {
    fn sign_strict(&self) -> Ordering;
}

pub trait PartialOrdStrict {
    fn partial_cmp_strict(&self, other: &Self) -> Option<Ordering>;
}

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

// Implementations

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

impl Ten for Mpn {
    const TEN: Self = Mpn::const_from(10);
}

impl Ten for Mpz {
    const TEN: Self = Mpz::const_from_unsigned(10);
}

impl Ten for Mpq {
    const TEN: Self = Mpq::const_from_unsigned(10);
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
