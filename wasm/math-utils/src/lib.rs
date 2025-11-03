use std::cmp::Ordering;
use std::str::FromStr;

use anyhow::anyhow;
use flagset::{FlagSet, Flags, flags};
use malachite::base::num::arithmetic::traits::{
    Abs, BinomialCoefficient, ExtendedGcd, Factorial, Gcd, Pow as MpPow, Sign, UnsignedAbs,
};
use paste::paste;

use fraction::GenericFraction;
use malachite::{Integer as Mpz, Natural as Mpn, Rational as Mpq};
use num::complex::{Complex, Complex64 as c64, ComplexFloat};
use num::{One as NumOne, Zero as NumZero};
use num_prime::nt_funcs;
use puruspe::bessel;
use quaternion::Quaternion;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use special::LambertW;
use wasm_minimal_protocol::*;

use math_utils_proc_macro::define_func;

use crate::frac::{Approx, ExtendedNumber, FracData, MpqExt, SignStrict};

mod complex;
mod frac;
mod quat;

initiate_protocol!();

trait IntoWasmOutput {
    fn into_wasm_output(self) -> Vec<u8>;
}

trait FromWasmInput: Sized {
    fn from_wasm_input(input: &[u8]) -> Result<Self, anyhow::Error>;
}

macro_rules! impl_wasm_conversion_for_num {
    ($($t: ty), *) => {
        $(
            impl FromWasmInput for $t {
                fn from_wasm_input(input: &[u8]) -> Result<Self, anyhow::Error> {
                    Ok(Self::from_le_bytes(input.try_into().unwrap()))
                }
            }
            impl IntoWasmOutput for $t {
                fn into_wasm_output(self) -> Vec<u8> {
                    self.to_le_bytes().to_vec()
                }
            }
        )*
    };
}

macro_rules! impl_wasm_conversion_for_complex {
    ($t: ty, $n_bytes: expr) => {
        impl FromWasmInput for Complex<$t> {
            fn from_wasm_input(input: &[u8]) -> Result<Self, anyhow::Error> {
                let re = <$t>::from_le_bytes(input[..$n_bytes].try_into()?);
                let im = <$t>::from_le_bytes(input[$n_bytes..].try_into()?);
                Ok(Complex::new(re, im))
            }
        }
        impl IntoWasmOutput for Complex<$t> {
            fn into_wasm_output(self) -> Vec<u8> {
                let mut out: Vec<u8> = Vec::with_capacity($n_bytes * 2);
                out.extend_from_slice(self.re.to_le_bytes().as_ref());
                out.extend_from_slice(self.im.to_le_bytes().as_ref());
                out
            }
        }
    };
}

macro_rules! impl_wasm_conversion_serialize {
    ($($t: ty), *) => {
        $(
            impl FromWasmInput for $t {
                fn from_wasm_input(input: &[u8]) -> Result<Self, anyhow::Error> {
                    Ok(ciborium::de::from_reader(input)?)
                }
            }
            impl IntoWasmOutput for $t {
                fn into_wasm_output(self) -> Vec<u8> {
                    let mut out = Vec::new();
                    ciborium::ser::into_writer(&self, &mut out).unwrap();
                    out
                }
            }
        )*
    };
}

impl_wasm_conversion_for_num!(
    /*f128,*/ f64, f32, /*f16,*/ i128, i64, i32, i16, i8, u128, u64, u32, u16, u8
);
impl_wasm_conversion_for_complex!(f64, 8);
impl_wasm_conversion_for_complex!(f32, 4);
impl_wasm_conversion_serialize!(Mpz, Mpn, Mpq, MpqExt);

impl FromWasmInput for String {
    fn from_wasm_input(input: &[u8]) -> Result<Self, anyhow::Error> {
        Ok(Self::from_utf8(input.to_vec())?)
    }
}

impl IntoWasmOutput for String {
    fn into_wasm_output(self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl FromWasmInput for bool {
    fn from_wasm_input(input: &[u8]) -> Result<Self, anyhow::Error> {
        match input[0] {
            0 => Ok(false),
            _ => Ok(true),
        }
    }
}

impl IntoWasmOutput for bool {
    fn into_wasm_output(self) -> Vec<u8> {
        if self { [1u8].to_vec() } else { [0u8].to_vec() }
    }
}

impl FromWasmInput for Ordering {
    fn from_wasm_input(input: &[u8]) -> Result<Self, anyhow::Error> {
        Ok((input[0] as i8).cmp(&0))
    }
}

impl IntoWasmOutput for Ordering {
    fn into_wasm_output(self) -> Vec<u8> {
        [self as u8].to_vec()
    }
}

impl FromWasmInput for Option<Ordering> {
    fn from_wasm_input(input: &[u8]) -> Result<Self, anyhow::Error> {
        if input.len() == 0 {
            Ok(None)
        } else {
            Ok(Some((input[0] as i8).cmp(&0)))
        }
    }
}

impl IntoWasmOutput for Option<Ordering> {
    fn into_wasm_output(self) -> Vec<u8> {
        match self {
            None => [].to_vec(),
            Some(ord) => [ord as u8].to_vec(),
        }
    }
}

impl<T1, T2, T3> IntoWasmOutput for (T1, T2, T3)
where
    T1: serde::Serialize,
    T2: serde::Serialize,
    T3: serde::Serialize,
{
    fn into_wasm_output(self) -> Vec<u8> {
        let mut out = Vec::new();
        ciborium::ser::into_writer(&self, &mut out).unwrap();
        out
    }
}

impl<T> FromWasmInput for Vec<T>
where
    T: serde::de::DeserializeOwned,
{
    fn from_wasm_input(input: &[u8]) -> Result<Self, anyhow::Error> {
        Ok(ciborium::de::from_reader(input)?)
    }
}

impl<T> IntoWasmOutput for Vec<T>
where
    T: serde::Serialize,
{
    fn into_wasm_output(self) -> Vec<u8> {
        let mut out = Vec::new();
        ciborium::ser::into_writer(&self, &mut out).unwrap();
        out
    }
}

impl<T> FromWasmInput for GenericFraction<T>
where
    T: Clone + Copy + fraction::Integer + DeserializeOwned,
{
    fn from_wasm_input(input: &[u8]) -> Result<Self, anyhow::Error> {
        let frac: FracData<T> = ciborium::de::from_reader(input)?;
        Ok(frac.into())
    }
}

impl<T> IntoWasmOutput for GenericFraction<T>
where
    T: Clone + Copy + fraction::Integer + Serialize,
{
    fn into_wasm_output(self) -> Vec<u8> {
        let frac = FracData::from(self);
        let mut out = Vec::new();
        ciborium::ser::into_writer(&frac, &mut out).unwrap();
        out
    }
}

impl<T> FromWasmInput for Quaternion<T>
where
    T: DeserializeOwned,
{
    fn from_wasm_input(input: &[u8]) -> Result<Self, anyhow::Error> {
        Ok(ciborium::de::from_reader(input)?)
    }
}

impl<T> IntoWasmOutput for Quaternion<T>
where
    T: Serialize,
{
    fn into_wasm_output(self) -> Vec<u8> {
        let mut out = Vec::new();
        ciborium::ser::into_writer(&self, &mut out).unwrap();
        out
    }
}

impl<F> FromWasmInput for FlagSet<F>
where
    F: Flags,
    F::Type: FromWasmInput,
{
    fn from_wasm_input(input: &[u8]) -> Result<Self, anyhow::Error> {
        let flags: F::Type = F::Type::from_wasm_input(input)?;
        Self::new(flags).map_err(|_| anyhow!("invalid bits"))
    }
}

impl<F> IntoWasmOutput for FlagSet<F>
where
    F: Flags,
    F::Type: IntoWasmOutput,
{
    fn into_wasm_output(self) -> Vec<u8> {
        self.bits().into_wasm_output()
    }
}

// impl<T> IntoWasmOutput for T where T: serde::Serialize {
//     fn into_wasm_output(self) -> Vec<u8> {
//         let mut out = Vec::new();
//         ciborium::ser::into_writer(&self, &mut out).unwrap();
//         out
//     }
// }

macro_rules! define_float_method_func {
    ($method: ident) => {
        define_func!($method, |num: f64| num.$method());
    };
}

macro_rules! define_complex_method_func {
    ($method: ident) => {
        paste! {define_func!([<$method _complex>], |num: c64| num.$method());}
    };
}

macro_rules! define_method_func_with_complex {
    ($func_name: ident) => {
        define_float_method_func!($func_name);
        define_complex_method_func!($func_name);
    };
}

// Common Functions

define_complex_method_func!(sin);
define_complex_method_func!(cos);
define_complex_method_func!(tan);
define_complex_method_func!(sinh);
define_complex_method_func!(cosh);
define_complex_method_func!(tanh);
define_complex_method_func!(asin);
define_complex_method_func!(acos);
define_complex_method_func!(atan);
define_complex_method_func!(exp);
define_complex_method_func!(ln);
define_complex_method_func!(log2);
define_complex_method_func!(log10);
define_complex_method_func!(sqrt);
define_complex_method_func!(cbrt);

define_method_func_with_complex!(asinh);
define_method_func_with_complex!(acosh);
define_method_func_with_complex!(atanh);

// Special Functions

define_func!(gamma, |x: f64| scirs2_special::gamma(x));
define_func!(gamma_complex, |z: c64| scirs2_special::gamma_complex(z));
define_func!(digamma, |x: f64| scirs2_special::digamma(x));
define_func!(digamma_complex, |z: c64| scirs2_special::digamma_complex(z));
define_func!(erf, |x: f64| scirs2_special::erf(x));
define_func!(erf_complex, |z: c64| scirs2_special::erf_complex(z));
define_func!(beta, |x1: f64, x2: f64| scirs2_special::beta(x1, x2));
define_func!(beta_complex, |z1: c64, z2: c64| {
    scirs2_special::beta_complex(z1, z2)
});
define_func!(lambert_w, |x: f64| x.lambert_w0());
define_func!(zeta, |x: f64| scirs2_special::zeta(x), true);
define_func!(zeta_complex, |z: c64| spfunc::zeta::zeta(z));
define_func!(airy_ai, |x: f64| scirs2_special::ai(x));
define_func!(airy_ai_complex, |x: c64| scirs2_special::ai_complex(x));
define_func!(airy_bi, |x: f64| scirs2_special::bi(x));
define_func!(airy_bi_complex, |x: c64| scirs2_special::bi_complex(x));
define_func!(bessel_jn, |n: i64, x: f64| bessel::Jn(n as u32, x));
define_func!(bessel_yn, |n: i64, x: f64| bessel::Yn(n as u32, x));

// Number Theory

#[wasm_func]
fn prime_factors(arg: &[u8]) -> Vec<u8> {
    let num = u64::from_le_bytes(arg.try_into().unwrap());
    let factor_repr = prime_factorization::Factorization::run(num);
    let mut out = Vec::new();
    ciborium::ser::into_writer(&factor_repr.factors, &mut out).unwrap();
    out
}

define_func!(extended_gcd, |m: i64, n: i64| ExtendedGcd::extended_gcd(
    m, n
));
define_func!(nth_prime, |n: u64| nt_funcs::nth_prime(n));
define_func!(prime_pi, |n: u64| nt_funcs::prime_pi(n));

// Rational / Fraction

#[allow(non_camel_case_types)]
type q64 = fraction::Fraction;

define_func!(
    parse_fraction,
    |src: String| {
        let myfrac = frac::Frac::<u64>::from_str(
            &src.replace("\u{2212}", "-")
                .replace("oo", "inf")
                .replace("\u{221E}", "inf"),
        )?;
        Ok::<q64, anyhow::Error>(myfrac.into())
    },
    true,
);
define_func!(fraction_from_float, |num: f64| q64::from(num));
define_func!(fraction_sub, |x: q64, y: q64| x - y);
define_func!(fraction_div, |x: q64, y: q64| x / y);
define_func!(fraction_cmp, |x: q64, y: q64| x.cmp(&y));
define_func!(fraction_approx, |x: q64, max_den: u64| q64::from(
    frac::Frac::<u64>::from(x).approx(&max_den)
));

#[wasm_func]
fn fraction_add(arg: &[u8]) -> Vec<u8> {
    let fracs: Vec<frac::FracData<u64>> = ciborium::de::from_reader(arg).unwrap();
    let result: q64 = fracs
        .iter()
        .map(|f| (*f).into())
        .fold(q64::zero(), |acc, x: q64| acc + x);
    let result = frac::FracData::from(result);
    let mut out = Vec::new();
    ciborium::ser::into_writer(&result, &mut out).unwrap();
    out
}

#[wasm_func]
fn fraction_mul(arg: &[u8]) -> Vec<u8> {
    let fracs: Vec<frac::FracData<u64>> = ciborium::de::from_reader(arg).unwrap();
    let result: q64 = fracs
        .iter()
        .map(|f| (*f).into())
        .fold(q64::one(), |acc, x: q64| acc * x);
    let result = frac::FracData::from(result);
    let mut out = Vec::new();
    ciborium::ser::into_writer(&result, &mut out).unwrap();
    out
}

define_func!(fraction_pow, |frac: q64, exp: i64| q64::from(
    frac::Frac::<u64>::from(frac).pow(exp)
),);

// Complex

fn decode_complex_seq(arg: &[u8]) -> impl Iterator<Item = c64> {
    arg.chunks_exact(16).map(|it| {
        let re = f64::from_le_bytes(it[..8].try_into().unwrap());
        let im = f64::from_le_bytes(it[8..].try_into().unwrap());
        c64::new(re, im)
    })
}

define_func!(
    parse_complex,
    |src: String| Ok::<c64, anyhow::Error>(c64::from_str(&src.replace("\u{2212}", "-"))?),
    true,
);

#[wasm_func]
fn complex_add(arg: &[u8]) -> Vec<u8> {
    let result: c64 = decode_complex_seq(arg).sum();
    result.into_wasm_output()
}

#[wasm_func]
fn complex_mul(arg: &[u8]) -> Vec<u8> {
    let result: c64 = decode_complex_seq(arg).product();
    result.into_wasm_output()
}

define_func!(complex_div, |z1: c64, z2: c64| z1 / z2);
define_func!(complex_pow_real, |z: c64, exp: f64| z.powf(exp));
define_func!(complex_pow_complex, |z1: c64, z2: c64| z1.powc(z2));
define_func!(complex_reci, |z: c64| z.recip());

// Quaternions

#[allow(non_camel_case_types)]
type h64 = Quaternion<f64>;

define_func!(quaternion_mul, |x: h64, y: h64| quaternion::mul(x, y));
// define_func!(quaternion_inv, |x: h64| quaternion::inv(x));

// Multi-precision Integers

define_func!(
    parse_mpz,
    |src: String| {
        malachite::Integer::from_str(&src.replace("\u{2212}", "-"))
            .map_err(|_| anyhow!("Invalid number format"))
    },
    true,
);
define_func!(mpz_from_int, |src: i64| Mpz::from(src));
define_func!(mpz_to_string, |x: Mpz| x.to_string());

#[wasm_func]
fn verify_mpz(arg: &[u8]) -> Vec<u8> {
    ciborium::de::from_reader::<Mpz, &[u8]>(arg)
        .is_ok()
        .into_wasm_output()
}

define_func!(mpz_add, |nums: Vec<Mpz>| nums.iter().sum::<Mpz>());
define_func!(mpz_sub, |x: Mpz, y: Mpz| x - y);
define_func!(mpz_mul, |nums: Vec<Mpz>| nums.iter().product::<Mpz>());
define_func!(mpz_div, |x: Mpz, y: Mpz| x / y);
define_func!(mpz_neg, |x: Mpz| -x);
define_func!(mpz_pow, |x: Mpz, y: u64| Mpz::pow(x, y));
define_func!(mpz_abs, |x: Mpz| x.unsigned_abs());
define_func!(mpz_sign, |x: Mpz| x.sign());
define_func!(mpz_cmp, |x: Mpz, y: Mpz| x.cmp(&y));
define_func!(mpz_fact, |n: u64| Mpn::factorial(n));
define_func!(mpz_binom, |n: Mpz, k: Mpz| Mpz::binomial_coefficient(n, k));
define_func!(mpz_gcd, |m: Mpz, n: Mpz| Mpn::gcd(
    m.unsigned_abs(),
    n.unsigned_abs()
));
define_func!(mpz_egcd, |m: Mpz, n: Mpz| Mpz::extended_gcd(m, n));

// Multi-precision Rationals

define_func!(
    parse_mpq,
    |src: String| {
        MpqExt::from_str(
            &src.replace("\u{2212}", "-")
                .replace("oo", "inf")
                .replace("\u{221E}", "inf"),
        )
        .map_err(|_| anyhow!("Invalid number format"))
    },
    true
);
define_func!(mpq_from_int, |n: i64| MpqExt::from(n));
define_func!(mpq_from_float, |n: f64| MpqExt::try_from(n), true);
define_func!(mpq_from_mpz, |n: Mpz| MpqExt::from(n));
define_func!(mpq_from_mpz_pair, |n: Mpz, d: Mpz| MpqExt::from_integers(
    n, d
));
define_func!(mpq_num, |x: MpqExt| x.into_numerator());
define_func!(mpq_den, |x: MpqExt| x.into_denominator());
define_func!(mpq_num_signed, |x: MpqExt| x.into_numerator_signed());
define_func!(mpq_den_signed, |x: MpqExt| x.into_denominator_signed());

#[wasm_func]
fn verify_mpq(arg: &[u8]) -> Vec<u8> {
    ciborium::de::from_reader::<MpqExt, &[u8]>(arg)
        .is_ok()
        .into_wasm_output()
}

define_func!(mpq_add, |nums: Vec<MpqExt>| nums.iter().sum::<MpqExt>());
define_func!(mpq_sub, |x: MpqExt, y: MpqExt| x - y);
define_func!(mpq_mul, |nums: Vec<MpqExt>| nums.iter().product::<MpqExt>());
define_func!(mpq_div, |x: MpqExt, y: MpqExt| x / y);
define_func!(mpq_neg, |x: MpqExt| -x);
define_func!(mpq_pow, |x: MpqExt, y: i64| MpqExt::pow(x, y));
define_func!(mpq_abs, |x: MpqExt| x.abs());
define_func!(mpq_sign, |x: MpqExt| x.sign());
define_func!(mpq_sign_strict, |x: MpqExt| x.sign_strict());
define_func!(mpq_repr, |x: MpqExt| x.to_string());
define_func!(
    mpq_to_str,
    |x: MpqExt, options: FlagSet<FracLayoutOptions>| { x.to_layout_string(options) }
);
define_func!(
    mpq_to_math,
    |x: MpqExt, options: FlagSet<FracLayoutOptions>| { x.to_math_strings(options) }
);
define_func!(mpq_cmp, |x: MpqExt, y: MpqExt| x.partial_cmp(&y));
define_func!(mpq_cmp_strict, |x: MpqExt, y: MpqExt| x
    .partial_cmp_strict(&y));
define_func!(mpq_is_finite, |x: MpqExt| x.is_finite());
define_func!(mpq_is_infinite, |x: MpqExt| x.is_infinite());
define_func!(mpq_is_nan, |x: MpqExt| x.is_nan());
define_func!(mpq_approx, |x: MpqExt, max_den: Mpn| x.approx(&max_den));

flags! {
    pub enum FracLayoutOptions: u8 {
        PlusSign,
        SignedZero,
        SignedInf,
        DenomOne,
        HyphenMinus,
    }
}

pub trait ToLayoutString {
    type Options;
    fn to_layout_string(&self, options: Self::Options) -> String;
}

macro_rules! minus_sign {
    ($b: expr) => {
        (if $b { '-' } else { '\u{2212}' })
    };
}

impl ToLayoutString for MpqExt {
    type Options = FlagSet<FracLayoutOptions>;

    fn to_layout_string(&self, options: Self::Options) -> String {
        use FracLayoutOptions::*;
        use MpqExt::*;

        let plus_sign = options.contains(PlusSign);
        let signed_zero = options.contains(SignedZero);
        let signed_inf = options.contains(SignedInf);
        let denom_one = options.contains(DenomOne);
        let hyphen_minus = options.contains(HyphenMinus);

        match self {
            NaN => "NaN".to_string(),
            &Zero(s) => {
                let mut out = String::with_capacity(if denom_one { 4 } else { 2 });
                if signed_zero {
                    if s {
                        if plus_sign {
                            out.push('+');
                        }
                    } else {
                        out.push(minus_sign!(hyphen_minus));
                    }
                }
                if denom_one {
                    out += "0/1";
                } else {
                    out.push('0');
                }
                out
            }
            &Inf(s) => {
                let mut out = String::with_capacity(2);
                if s {
                    if plus_sign | signed_inf {
                        out.push('+');
                    }
                } else {
                    out.push(minus_sign!(hyphen_minus));
                }
                out.push('\u{221E}');
                out
            }
            Rational(q) => {
                let mut out = String::with_capacity(10);
                use Ordering::*;
                match q.sign() {
                    Less => out.push(minus_sign!(hyphen_minus)),
                    Greater => {
                        if plus_sign {
                            out.push('+');
                        }
                    }
                    Equal => unreachable!(),
                }
                out += &(q.numerator_ref().to_string());
                if !denom_one & (q.denominator_ref() == &1) {
                    return out;
                } else {
                    out.push('/');
                    out += &(q.denominator_ref().to_string());
                }
                out
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct ToMathStringResult {
    sign: Option<char>,
    num: String,
    den: Option<String>,
}
impl_wasm_conversion_serialize!(ToMathStringResult);

impl MpqExt {
    fn to_math_strings(&self, options: FlagSet<FracLayoutOptions>) -> ToMathStringResult {
        use FracLayoutOptions::*;
        use MpqExt::*;

        let plus_sign = options.contains(PlusSign);
        let signed_zero = options.contains(SignedZero);
        let signed_inf = options.contains(SignedInf);
        let denom_one = options.contains(DenomOne);

        match self {
            NaN => ToMathStringResult {
                sign: None,
                num: "NaN".to_string(),
                den: None,
            },
            &Zero(s) => {
                // let sign = if s {
                //     if plus_sign { Some('+') } else { None }
                // } else {
                //     if signed_zero { Some('\u{2212}') } else { None }
                // };
                let sign = if signed_zero {
                    if s {
                        if plus_sign { Some('+') } else { None }
                    } else {
                        Some('\u{2212}')
                    }
                } else {
                    None
                };
                let denominator = if denom_one {
                    Some("1".to_string())
                } else {
                    None
                };
                ToMathStringResult {
                    sign,
                    num: '0'.to_string(),
                    den: denominator,
                }
            }
            &Inf(s) => {
                let sign = if s {
                    if plus_sign | signed_inf {
                        Some('+')
                    } else {
                        None
                    }
                } else {
                    Some('\u{2212}')
                };
                ToMathStringResult {
                    sign,
                    num: '\u{221E}'.to_string(),
                    den: None,
                }
            }
            Rational(q) => {
                use Ordering::*;
                let sign = match q.sign() {
                    Less => Some('\u{2212}'),
                    Greater => {
                        if plus_sign {
                            Some('+')
                        } else {
                            None
                        }
                    }
                    Equal => unreachable!(),
                };
                let numerator = q.numerator_ref().to_string();
                let denominator = if !denom_one & (q.denominator_ref() == &1) {
                    None
                } else {
                    Some(q.denominator_ref().to_string())
                };
                ToMathStringResult {
                    sign,
                    num: numerator,
                    den: denominator,
                }
            }
        }
    }
}
