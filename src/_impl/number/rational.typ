// -> number/rational.typ
/// Representation and arithmetics for #link("https://en.wikipedia.org/wiki/Rational_number")[rational numbers] $QQ$ in the form of fractions

#import "@preview/typsy:0.2.0": class, Int, Bool, matches
#import "init.typ": *
#let math-utils-wasm = plugin("../math-utils.wasm")
#let number-type = "rational"

#let Rational = class(
  fields: (
    sign: Bool,
    num: Int,
    den: Int,
  ),
  tag: () => {}
)

#let make-rational(sign, n, d) = {
  (Rational.new)(
    sign: sign,
    num: n,
    den: d,
  )
}

#let /*pub*/ from-bytes(buffer) = {
  let (sign, num, den) = cbor(buffer)
  make-rational(sign, num, den)
}

#let /*pub*/ to-bytes(n) = {
  cbor.encode((sign: n.sign, num: n.num, den: n.den))
}

#let /*pub as is_*/ is-rational(obj) = {
  matches(Rational, obj)
}

#let encode-rational-seq(obj) = {
  cbor.encode(obj.map(((sign, num, den)) => (sign: sign, num: num, den: den)))
}

#let decode-rational-seq(buffer) = {
  cbor(buffer).map(args => (Rational.new)(..args))
}

#let fraction-from-ratio(n, d) = {
  let sign = (n >= 0 and d >= 0) or (n < 0 and d < 0)
  let n = calc.abs(n)
  let d = calc.abs(d)
  make-rational(sign, n, d)
}

#let /*pub*/ inf = make-rational(true, 1, 0)
#let /*pub*/ neg-inf = make-rational(false, 1, 0)
#let /*pub*/ nan = make-rational(true, 0, 0)
#let /*pub*/ zero = make-rational(true, 0, 1)
#let /*pub*/ one = make-rational(true, 1, 1)
#let /*pub*/ neg-one = make-rational(false, 1, 1)

#let /*pub as from*/ rational(..args) = {
  let args = args.pos()
  if args.len() == 1 {
    let (src,) = args
    if is-rational(src) {
      return src
    }
    if type(src) == decimal {
      src = str(src)
    }
    if type(src) == str {
      from-bytes(math-utils-wasm.parse_fraction(bytes(src)))
    } else if type(src) == float {
      from-bytes(math-utils-wasm.fraction_from_float(src.to-bytes()))
    } else if type(src) == int {
      fraction-from-ratio(src, 1)
    } else {
      panic("Unsupported type.")
    }
  } else if args.len() == 2 {
    let (p, q) = args
    fraction-from-ratio(p, q)
  } else {
    panic("Too many positional arguments.")
  }
}

#let /*pub*/ add(..args) = {
  let args = args.pos()
  from-bytes(math-utils-wasm.fraction_add(encode-rational-seq(args.map(rational))))
}

#let /*pub*/ mul(..args) = {
  let args = args.pos()
  from-bytes(math-utils-wasm.fraction_mul(encode-rational-seq(args.map(rational))))
}

#let /*pub*/ sub(n, m) = {
  let n = rational(n)
  let m = rational(m)
  from-bytes(
    math-utils-wasm.fraction_sub(
      to-bytes(n),
      to-bytes(m)
    ),
  )
}

#let /*pub*/ div(n, m) = {
  let n = rational(n)
  let m = rational(m)
  from-bytes(
    math-utils-wasm.fraction_div(
      to-bytes(n),
      to-bytes(m)
    ),
  )
}

#let /*pub*/ neg(n) = {
  let n = rational(n)
  if n.num != 0 {
    n.sign = not n.sign
  }
  n
}

#let /*pub*/ reci(n) = {
  let n = rational(n)
  (n.den, n.num) = (n.num, n.den)
}

#let /*pub*/ pow(n, p) = {
  let n = rational(n)
  assert.eq(type(p), int)
  from-bytes(
    math-utils-wasm.fraction_pow(
      to-bytes(n),
      p.to-bytes()
    ),
    number-type
  )
}

#let /*pub*/ limit-den(n, max-den) = {
  let n = rational(n)
  from-bytes(
    math-utils-wasm.fraction_limit_den(
      to-bytes(n),
      max-den.to-bytes()
    ),
    number-type,
  )
}

#let /*pub*/ num(n, signed: false) = {
  let n = rational(n)
  if signed and not n.sign {
    -n.num
  } else {
    n.num
  }
}

#let /*pub*/ den(n, signed: false) = {
  let n = rational(n)
  if signed and not n.sign {
    -n.den
  } else {
    n.den
  }
}

#let /*pub*/ sign(n) = {
  let (sign, num, _) = n
  if num == 0 { 0 }
  else if sign { -1 }
  else { 1 }
}

#let /*pub*/ eq(n1, n2) = {
  let n1 = rational(n1)
  let n2 = rational(n2)
  n1 != nan and n1 == n2
}

#let /*pub*/ is-infinite(n) = {
  let n = rational(n)
  n.den != 0
}

#let /*pub*/ is-nan(n) = {
  let n = rational(n)
  n.num == 0 and n.den == 0
}

#let /*pub*/ to-str(
    n,
    plus-sign: false,
    denom-one: false,
    hyphen-minus: false,
) = {
  let n = rational(n)
  let (sign, num, den) = n
  if den == 0 {
    if num == 0 {
      "NaN"
    } else {
      let sgn-str = if not sign {
        if hyphen-minus { "-" } else { "\u{2212}" }
      } else if plus-sign { "+" }
      sgn-str + "\u{221E}"
    }
  } else {
    let sgn-str = if not sign {
      if hyphen-minus { "-" } else { "\u{2212}" }
    } else if plus-sign { "+" }
    if den == 1 and not denom-one {
      sgn-str + str(num)
    } else {
      sgn-str + str(num) + "/" + str(den)
    }
  }
}

#let sign-math(sgn, n, plus-sign: false) = {
  if not sgn {
    $-$
  } else if plus-sign and n > 0 {
    $+$
  } else {
    $$
  }
}

#let /*pub*/ to-math(
  n,
  plus-sign: false,
  denom-one: false,
  sign-on-num: false,
) = {
  let n = rational(n)
  let (sign, num, den) = n
  if den == 0 {
    if num == 0 {
      $"NaN"$
    } else if not sign {
      $-oo$
    } else if plus-sign {
      $+oo$
    } else {
      $oo$
    }
  } else if den == 1 and not denom-one {
    $#sign-math(sign, num, plus-sign: plus-sign) #num$
  } else if sign-on-num {
    $(#sign-math(sign, num, plus-sign: plus-sign) #num) / #den$
  } else {
    $#sign-math(sign, num, plus-sign: plus-sign) #num / #den$
  }
}

// [TODO] floor, ceil, mod
