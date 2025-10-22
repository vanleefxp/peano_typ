// -> number/rational/arith.typ

#let math-utils-wasm = plugin("../../math-utils.wasm")
#import "init.typ": rational, from-bytes, to-bytes, encode-rational-seq

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
