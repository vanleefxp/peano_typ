#import "number/complex.typ": complex, is-complex, make-complex
#let math-utils-wasm = dictionary(plugin("math-utils.wasm"))

#let define-special-func-with-complex(func-name) = {
  let real-func = math-utils-wasm.at(func-name)
  let complex-func = math-utils-wasm.at(func-name + "_complex")
  x => {
    if is-complex(x) {
      let result-bytes = complex-func(
        x.re.to-bytes(),
        x.im.to-bytes(),
      )
      let re = float.from-bytes(result-bytes.slice(0, 8))
      let im = float.from-bytes(result-bytes.slice(8))
      make-complex(re, im)
    } else {
      let x = float(x)
      float.from-bytes(
        real-func(x.to-bytes(endian: "little")),
        endian: "little",
      )
    }
  }
}

#let define-special-func-2-with-complex(func-name) = {
  let real-func = math-utils-wasm.at(func-name)
  let complex-func = math-utils-wasm.at(func-name + "_complex")
  (x1, x2) => {
    if is-complex(x1) or is-complex(x2) {
      let x1 = complex(x1)
      let x2 = complex(x2)
      let result-bytes = complex-func(
        x1.re.to-bytes(),
        x1.im.to-bytes(),
        x2.re.to-bytes(),
        x2.im.to-bytes(),
      )
      let re = float.from-bytes(result-bytes.slice(0, 8))
      let im = float.from-bytes(result-bytes.slice(8))
      make-complex(re, im)
    } else {
      let x1 = float(x1)
      let x2 = float(x2)
      float.from-bytes(real-func(x1.to-bytes(), x2.to-bytes()))
    }
  }
}

/// The #link("https://en.wikipedia.org/wiki/Gamma_function")[$Gamma$ function],
/// defined by $Gamma(z) = integral_0^oo t^(z - 1) upright(e)^(-t) dif t$.
#let gamma = define-special-func-with-complex("gamma")

/// The #link("https://en.wikipedia.org/wiki/Digamma_function")[digamma function],
/// which is the derivative of the logarithm of $Gamma$ function
/// $psi(z) = dif/(dif z) ln Gamma(z) = (Gamma'(z))/(Gamma(z))$.
#let digamma = define-special-func-with-complex("digamma")

/// The #link("https://en.wikipedia.org/wiki/Error_function")[Gauss error function],
/// defined by $erf z = 2/sqrt(pi) integral_0^z e^(-t^2) dif t$
#let erf = define-special-func-with-complex("erf")

/// #link("https://en.wikipedia.org/wiki/Riemann_zeta_function")[Riemann's $zeta$ function]
/// defined by $zeta(s) = sum_(n = 1)^oo 1/(n^s)$ for $Re s > 1$ and its analytic continuation otherwise.
#let zeta = define-special-func-with-complex("zeta")

/// The #link("https://en.wikipedia.org/wiki/Beta_function")[$Beta$ function],
/// defined by $Beta(z_1, z_2) = integral_0^1 t^(z_1 - 1) (1 - t)^(z_2 - 1) dif t$.
/// Equals to $(Gamma(z_1) Gamma(z_2))/(Gamma(z_1 + z_2))$
#let beta = define-special-func-2-with-complex("beta")

// Euler's $gamma$ constant. Equals to $lim_(n -> oo) ((sum_(k = 1)^(n) 1/n) - ln n)$
#let euler-gamma = -digamma(1)
