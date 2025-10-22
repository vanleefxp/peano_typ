// -> number/complex/arith.typ

#import "init.typ": complex, from-bytes, encode-complex-seq
#let math-utils-wasm = plugin("../../math-utils.wasm")

#let /*pub*/ abs(z) = {
  let (re, im) = complex(z)
  calc.norm(re, im)
}

#let /*pub*/ arg(z) = {
  let (re, im) = complex(z)
  calc.atan2(re, im)
}

#let /*pub*/ conj(z) = {
  let (re, im) = complex(z)
  make-complex(re, -im)
}

#let /*pub*/ neg(z) = {
  let (re, im) = complex(z)
  make-complex(-re, -im)
}

#let /*pub*/ re(z) = {
  complex(z).re
}

#let /*pub*/ im(z) = {
  complex(z).im
}

#let /*pub*/ add(..args) = {
  if args.pos().len() == 0 {
    zero
  } else {
    from-bytes(
      math-utils-wasm.complex_add(encode-complex-seq(args.pos())),
    )
  }
}

#let /*pub*/ mul(..args) = {
  if args.pos().len() == 0 {
    one
  } else {
    from-bytes(
      math-utils-wasm.complex_mul(encode-complex-seq(args.pos())),
    )
  }
}

#let /*pub*/ sub(z1, z2) = {
  let z1 = complex(z1)
  let z2 = complex(z2)
  make-complex(z1.re - z2.re, z1.im - z2.im)
}

#let /*pub*/ div(z1, z2) = {
  from-bytes(
    math-utils-wasm.complex_div(
      to-bytes(z1),
      to-bytes(z2),
    ),
  )
}

#let /*pub*/ pow(z1, z2) = {
  let z1 = complex(z1)
  if type(z2) == int or type(z2) == decimal {
    z2 = float(z2)
  }
  if type(z2) == float {
    from-bytes(
      math-utils-wasm.complex_pow_real(
        to-bytes(z1),
        z2.to-bytes(),
      ),
    )
  } else {
    from-bytes(
      math-utils-wasm.complex_pow_complex(
        to-bytes(z1),
        to-bytes(z2),
      ),
    )
  }
}

#let /*pub*/ eq(z1, z2) = {
  let z1 = complex(z1)
  let z2 = complex(z2)
  z1 == z2
}
