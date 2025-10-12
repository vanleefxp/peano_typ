use num::complex::Complex64;
use std::num::ParseFloatError;
use thiserror::Error;

const COMPLEX_SYMBOL: char = 'i';
const SIGNS: [char; 2] = ['-', '+'];

#[derive(Debug, Error)]
pub enum ParseComplexError {
    #[error("Invalid complex number format")]
    InvalidFormat,
    #[error("Invalid float format")]
    ParseFloatError(#[from] ParseFloatError),
}

pub fn parse_complex(src: &str) -> Result<Complex64, ParseComplexError> {
    // see if the string starts with a sign
    let start: usize = if src.starts_with(&SIGNS) { 1 } else { 0 };
    // see if the string has imaginary part
    // by trying to find the sign before the imaginary part
    let im_start = match src[start..].find(&SIGNS) {
        Some(i) => {
            let sign_idx = start + i;
            match src.chars().nth(sign_idx - 1) {
                Some('e') | Some('E') => {
                    // a sign before float exponent
                    // need to find a sign afterwards
                    match src[sign_idx + 1..].find(&SIGNS) {
                        Some(j) => {
                            let sign_idx = sign_idx + j + 1;
                            match src.chars().nth(sign_idx - 1) {
                                // another sign before exponent
                                // not a sign before imaginary part
                                Some('e') | Some('E') => None,
                                // found a sign before imaginary part
                                _ => Some(sign_idx),
                            }
                        }
                        None => None,
                    }
                }
                _ => Some(sign_idx),
            }
        }
        None => None,
    };
    match im_start {
        // with both real and imaginary parts
        Some(i) => {
            if src.ends_with(COMPLEX_SYMBOL) {
                let re: f64 = src[..i]
                    .parse()
                    .map_err(ParseComplexError::ParseFloatError)?;
                let im: f64 = src[i..src.len() - 1]
                    .parse()
                    .map_err(ParseComplexError::ParseFloatError)?;
                Ok(Complex64::new(re, im))
            } else {
                // not a valid complex number format if the string doesn't end with 'i'
                Err(ParseComplexError::InvalidFormat)
            }
        }
        None => {
            if src.ends_with(COMPLEX_SYMBOL) {
                // with only imaginary part
                let im = src[..src.len() - 1]
                    .parse()
                    .map_err(ParseComplexError::ParseFloatError)?;
                Ok(Complex64::new(0.0, im))
            } else {
                // with only real part
                let re = src.parse().map_err(ParseComplexError::ParseFloatError)?;
                Ok(Complex64::new(re, 0.0))
            }
        }
    }
}
