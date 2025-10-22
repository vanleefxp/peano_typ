use paste::paste;

use fraction::Sign;
use num::complex::Complex64;
use num::integer::Integer;
use num::pow::Pow;
use num::{One, Zero};
use num_prime::nt_funcs;
use wasm_minimal_protocol::*;
use puruspe::bessel;

mod complex;
mod frac;

initiate_protocol!();

trait IntoWasmOutput {
    fn into_wasm_output(self) -> Vec<u8>;
}

impl IntoWasmOutput for f64 {
    fn into_wasm_output(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl IntoWasmOutput for u64 {
    fn into_wasm_output(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl IntoWasmOutput for Complex64 {
    fn into_wasm_output(self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(16);
        out.extend_from_slice(self.re.to_le_bytes().as_ref());
        out.extend_from_slice(self.im.to_le_bytes().as_ref());
        out
    }
}

macro_rules! define_func {
    ($func_name: ident, $arg_type: ty, $calc_expr: expr) => {
        #[wasm_func]
        fn $func_name(arg: &[u8]) -> Vec<u8> {
            let num = <$arg_type>::from_le_bytes(arg.try_into().unwrap());
            let result = $calc_expr(num);
            result.into_wasm_output()
        }
    };
}

macro_rules! define_failable_func {
    ($func_name: ident, $arg_type: ty, $calc_expr: expr) => {
        #[wasm_func]
        fn $func_name(arg: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
            let num = <$arg_type>::from_le_bytes(arg.try_into().unwrap());
            let result = $calc_expr(num)?;
            Ok(result.into_wasm_output())
        }
    };
}

macro_rules! define_complex_func {
    ($func_name: ident, $calc_expr: expr) => {
        #[wasm_func]
        fn $func_name(arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
            let num = decode_complex(arg1, arg2);
            let result = $calc_expr(num);
            result.into_wasm_output()
        }
    };
}

macro_rules! define_float_method_func {
    ($method: ident) => {
        define_func!($method, f64, |num: f64| num.$method());
    };
}

macro_rules! define_complex_method_func {
    ($method: ident) => {
        paste! {
            #[wasm_func]
            fn [<$method _complex>](arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
                let num = decode_complex(arg1, arg2);
                let result = num.$method();
                result.into_wasm_output()
            }
        }
    };
}

macro_rules! define_method_func_with_complex {
    ($func_name: ident) => {
        define_float_method_func!($func_name);
        define_complex_method_func!($func_name);
    };
}

// Special Functions

macro_rules! define_special_func {
    ($func_name: ident) => {
        #[wasm_func]
        fn $func_name(arg: &[u8]) -> Vec<u8> {
            let x = f64::from_le_bytes(arg.try_into().unwrap());
            let y = scirs2_special::$func_name(x);
            y.into_wasm_output()
        }
    };
}

macro_rules! define_special_func_2 {
    ($func_name: ident) => {
        #[wasm_func]
        fn $func_name(arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
            let x1 = f64::from_le_bytes(arg1.try_into().unwrap());
            let x2 = f64::from_le_bytes(arg2.try_into().unwrap());
            let y = scirs2_special::$func_name(x1, x2);
            y.into_wasm_output()
        }
    };
}

macro_rules! define_special_func_with_complex {
    ($func_name: ident) => {
        define_special_func!($func_name);

        paste! {
            #[wasm_func]
            fn [< $func_name _complex >](arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
                let re = f64::from_le_bytes(arg1.try_into().unwrap());
                let im = f64::from_le_bytes(arg2.try_into().unwrap());
                let z = Complex64::new(re, im);
                let y = scirs2_special::[<$func_name _complex >](z);
                y.into_wasm_output()
            }
        }
    };
}

macro_rules! define_special_func_2_with_complex {
    ($func_name: ident) => {
        define_special_func_2!($func_name);

        paste! {
            #[wasm_func]
            fn [< $func_name _complex >](arg1: &[u8], arg2: &[u8], arg3: &[u8], arg4: &[u8]) -> Vec<u8> {
                let re1 = f64::from_le_bytes(arg1.try_into().unwrap());
                let im1 = f64::from_le_bytes(arg2.try_into().unwrap());
                let re2 = f64::from_le_bytes(arg3.try_into().unwrap());
                let im2 = f64::from_le_bytes(arg4.try_into().unwrap());
                let z1 = Complex64::new(re1, im1);
                let z2 = Complex64::new(re2, im2);
                let y = scirs2_special::[< $func_name _complex >](z1, z2);
                y.into_wasm_output()
            }
        }
    };
}

define_special_func_with_complex!(gamma);
define_special_func_with_complex!(digamma);
define_special_func_with_complex!(erf);
define_special_func_2_with_complex!(beta);
define_failable_func!(zeta, f64, |x: f64| scirs2_special::zeta(x));
define_complex_func!(zeta_complex, |z: Complex64| spfunc::zeta::zeta(z));

define_func!(airy_ai, f64, |x: f64| scirs2_special::ai(x));
define_complex_func!(airy_ai_complex, |x: Complex64| {
    scirs2_special::ai_complex(x)
});
define_func!(airy_bi, f64, |x: f64| scirs2_special::bi(x));
define_complex_func!(airy_bi_complex, |x: Complex64| {
    scirs2_special::bi_complex(x)
});

#[wasm_func]
fn bessel_jn(arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
    let n = u32::from_le_bytes(arg1.try_into().unwrap());
    let x = f64::from_le_bytes(arg2.try_into().unwrap());
    let result = bessel::Jn(n, x);
    result.into_wasm_output()
}

#[wasm_func]
fn bessel_in(arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
    let n = u32::from_le_bytes(arg1.try_into().unwrap());
    let x = f64::from_le_bytes(arg2.try_into().unwrap());
    let result = bessel::Yn(n, x);
    result.into_wasm_output()
}

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

define_func!(nth_prime, u64, |n: u64| nt_funcs::nth_prime(n));
define_func!(prime_pi, u64, |n: u64| nt_funcs::prime_pi(n));

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

fn decode_complex(arg1: &[u8], arg2: &[u8]) -> Complex64 {
    let re = f64::from_le_bytes(arg1.try_into().unwrap());
    let im = f64::from_le_bytes(arg2.try_into().unwrap());
    Complex64::new(re, im)
}

fn decode_complex_seq(arg: &[u8]) -> impl Iterator<Item = Complex64> {
    arg.chunks_exact(16).map(|it| {
        let re = f64::from_le_bytes(it[..8].try_into().unwrap());
        let im = f64::from_le_bytes(it[8..].try_into().unwrap());
        Complex64::new(re, im)
    })
}

#[wasm_func]
fn parse_complex(arg: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let src = String::from_utf8(arg.to_vec())
        .unwrap()
        .replace("\u{2212}", "-");
    let num: Complex64 = str::parse(&src)?;
    Ok(num.into_wasm_output())
}

#[wasm_func]
fn complex_add(arg: &[u8]) -> Vec<u8> {
    let result: Complex64 = decode_complex_seq(arg).sum();
    result.into_wasm_output()
}

#[wasm_func]
fn complex_mul(arg: &[u8]) -> Vec<u8> {
    let result: Complex64 = decode_complex_seq(arg).product();
    result.into_wasm_output()
}

#[wasm_func]
fn complex_div(arg1: &[u8], arg2: &[u8], arg3: &[u8], arg4: &[u8]) -> Vec<u8> {
    let num1 = decode_complex(arg1, arg2);
    let num2 = decode_complex(arg3, arg4);
    let result = num1 / num2;
    result.into_wasm_output()
}

#[wasm_func]
fn complex_pow_real(arg1: &[u8], arg2: &[u8], arg3: &[u8]) -> Vec<u8> {
    let base = decode_complex(arg1, arg2);
    let exp = f64::from_le_bytes(arg3.try_into().unwrap());
    let result = base.powf(exp);
    result.into_wasm_output()
}

#[wasm_func]
fn complex_pow_complex(arg1: &[u8], arg2: &[u8], arg3: &[u8], arg4: &[u8]) -> Vec<u8> {
    let base = decode_complex(arg1, arg2);
    let exp = decode_complex(arg3, arg4);
    let result = base.powc(exp);
    result.into_wasm_output()
}

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
