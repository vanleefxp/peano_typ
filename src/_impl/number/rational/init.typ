// -> number/rational/init.typ

#import "@preview/typsy:0.2.0": class, Bool, Int, matches
#let math-utils-wasm = plugin("../../math-utils.wasm")

#let Rational = class(
  name: "rational",
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
