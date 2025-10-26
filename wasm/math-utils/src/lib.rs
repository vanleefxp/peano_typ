use paste::paste;

use fraction::Sign;
use num::complex::{Complex, Complex64 as c64};
use num::integer::Integer;
use num::pow::Pow;
use num::{One, Zero};
use num_prime::nt_funcs;
use puruspe::bessel;
use special::LambertW;
use wasm_minimal_protocol::*;

use math_utils_proc_macro::define_func;

mod complex;
mod frac;

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

impl_wasm_conversion_for_num!(
    /*f128,*/ f64, f32, /*f16,*/ i128, i64, i32, i16, i8, u128, u64, u32, u16, u8
);
impl_wasm_conversion_for_complex!(f64, 8);
impl_wasm_conversion_for_complex!(f32, 4);

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
define_func!(zeta, |x: f64| scirs2_special::zeta(x).unwrap());
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

#[wasm_func]
fn extended_gcd(arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
    let a = i64::from_le_bytes(arg1.try_into().unwrap());
    let b = i64::from_le_bytes(arg2.try_into().unwrap());
    let result = i64::extended_gcd(&a, &b);
    let mut out = Vec::new();
    ciborium::ser::into_writer(&(result.gcd, result.x, result.y), &mut out).unwrap();
    out
}

define_func!(nth_prime, |n: u64| nt_funcs::nth_prime(n));
define_func!(prime_pi, |n: u64| nt_funcs::prime_pi(n));

// Rational / Fraction

type F = fraction::Fraction;

#[wasm_func]
fn fraction(arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
    let num = i64::from_le_bytes(arg1.try_into().unwrap());
    let den = i64::from_le_bytes(arg2.try_into().unwrap());
    let frac = frac::MyFrac::from(F::new_generic(Sign::Plus, num, den).unwrap());
    let mut out = Vec::new();
    ciborium::ser::into_writer(&frac, &mut out).unwrap();
    out
}

#[wasm_func]
fn parse_fraction(arg1: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let src = String::from_utf8(arg1.to_vec())
        .unwrap()
        .replace("\u{2212}", "-");
    // let frac = frac::MyFrac::from(F::from_str(src.as_str())?);
    let frac = frac::MyFrac::from(frac::parse_fraction::<u64>(&src)?);
    let mut out = Vec::new();
    ciborium::ser::into_writer(&frac, &mut out).unwrap();
    Ok(out)
}

#[wasm_func]
fn fraction_from_float(arg: &[u8]) -> Vec<u8> {
    let num = f64::from_le_bytes(arg.try_into().unwrap());
    let frac = frac::MyFrac::from(F::from(num));
    let mut out = Vec::new();
    ciborium::ser::into_writer(&frac, &mut out).unwrap();
    out
}

#[wasm_func]
fn fraction_sub(arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
    let frac1: frac::MyFrac<u64> = ciborium::de::from_reader(arg1).unwrap();
    let frac2: frac::MyFrac<u64> = ciborium::de::from_reader(arg2).unwrap();
    let frac1: F = frac1.into();
    let frac2: F = frac2.into();
    let result = frac::MyFrac::from(frac1 - frac2);
    let mut out = Vec::new();
    ciborium::ser::into_writer(&result, &mut out).unwrap();
    out
}

#[wasm_func]
fn fraction_div(arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
    let frac1: frac::MyFrac<u64> = ciborium::de::from_reader(arg1).unwrap();
    let frac2: frac::MyFrac<u64> = ciborium::de::from_reader(arg2).unwrap();
    let frac1: F = frac1.into();
    let frac2: F = frac2.into();
    let result = frac::MyFrac::from(frac1 / frac2);
    let mut out = Vec::new();
    ciborium::ser::into_writer(&result, &mut out).unwrap();
    out
}

#[wasm_func]
fn fraction_neg(arg: &[u8]) -> Vec<u8> {
    let frac: frac::MyFrac<u64> = ciborium::de::from_reader(arg).unwrap();
    let frac: F = frac.into();
    let result = frac::MyFrac::from(-frac);
    let mut out = Vec::new();
    ciborium::ser::into_writer(&result, &mut out).unwrap();
    out
}

#[wasm_func]
fn fraction_reci(arg: &[u8]) -> Vec<u8> {
    let frac: frac::MyFrac<u64> = ciborium::de::from_reader(arg).unwrap();
    let frac: F = frac.into();
    let result = frac::MyFrac::from(frac.recip());
    let mut out = Vec::new();
    ciborium::ser::into_writer(&result, &mut out).unwrap();
    out
}

#[wasm_func]
fn fraction_add(arg: &[u8]) -> Vec<u8> {
    let fracs: Vec<frac::MyFrac<u64>> = ciborium::de::from_reader(arg).unwrap();
    let result: F = fracs
        .iter()
        .map(|f| (*f).into())
        .fold(F::zero(), |acc, x: F| acc + x);
    let result = frac::MyFrac::from(result);
    let mut out = Vec::new();
    ciborium::ser::into_writer(&result, &mut out).unwrap();
    out
}

#[wasm_func]
fn fraction_mul(arg: &[u8]) -> Vec<u8> {
    let fracs: Vec<frac::MyFrac<u64>> = ciborium::de::from_reader(arg).unwrap();
    let result: F = fracs
        .iter()
        .map(|f| (*f).into())
        .fold(F::one(), |acc, x: F| acc * x);
    let result = frac::MyFrac::from(result);
    let mut out = Vec::new();
    ciborium::ser::into_writer(&result, &mut out).unwrap();
    out
}

#[wasm_func]
fn fraction_limit_den(arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
    let frac: frac::MyFrac<u64> = ciborium::de::from_reader(arg1).unwrap();
    let limit = u64::from_le_bytes(arg2.try_into().unwrap());
    let frac = frac.limit_den(limit);
    let mut out = Vec::new();
    ciborium::ser::into_writer(&frac, &mut out).unwrap();
    out
}

#[wasm_func]
fn fraction_pow(arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
    let frac: frac::MyFrac<u64> = ciborium::de::from_reader(arg1).unwrap();
    let exp = i64::from_le_bytes(arg2.try_into().unwrap());
    let frac = frac.pow(exp);
    let mut out = Vec::new();
    ciborium::ser::into_writer(&frac, &mut out).unwrap();
    out
}

// Complex

fn decode_complex_seq(arg: &[u8]) -> impl Iterator<Item = c64> {
    arg.chunks_exact(16).map(|it| {
        let re = f64::from_le_bytes(it[..8].try_into().unwrap());
        let im = f64::from_le_bytes(it[8..].try_into().unwrap());
        c64::new(re, im)
    })
}

#[wasm_func]
fn parse_complex(arg: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let src = String::from_utf8(arg.to_vec())
        .unwrap()
        .replace("\u{2212}", "-");
    let num: c64 = str::parse(&src)?;
    Ok(num.into_wasm_output())
}

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
