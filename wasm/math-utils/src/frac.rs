use fraction::{GenericFraction, Sign};
use num::integer::Integer;
use num::pow::Pow;
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
