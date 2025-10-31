#import "init.typ": mp-rational, to-bytes
#let math-utils-wasm = plugin("../../../math-utils.wasm")

#let /*pub*/ add(..args) = {
  mp-rational(
    buffer: math-utils-wasm.mpq_add(
      cbor.encode(args.pos().map(it => cbor(to-bytes((it)))))
    )
  )
}

#let /*pub*/ mul(..args) = {
  mp-rational(
    buffer: math-utils-wasm.mpq_mul(
      cbor.encode(args.pos().map(it => cbor(to-bytes((it)))))
    )
  )
}

#let /*pub*/ sub(m, n) = {
  mp-rational(buffer: math-utils-wasm.mpq_sub(to-bytes(m), to-bytes(n)))
}

#let /*pub*/ div(m, n) = {
  mp-rational(buffer: math-utils-wasm.mpq_div(to-bytes(m), to-bytes(n)))
}

#let /*pub*/ neg(n) = {
  mp-rational(buffer: math-utils-wasm.mpq_neg(to-bytes(n)))
}

#let /*pub*/ pow(n, p) = {
  mp-rational(buffer: math-utils-wasm.mpq_pow(to-bytes(n), int(p).to-bytes()))
}

#let /*pub*/ reci(n) = {
  mp-rational(buffer: math-utils-wasm.mpq_reci(to-bytes(n)))
}

#let /*pub*/ abs(n) = {
  mp-rational(buffer: math-utils-wasm.mpq_abs(to-bytes(n)))
}

#let /*pub*/ sign(n, strict: false) = {
  int.from-bytes(
    (
      if strict { math-utils-wasm.mpq_sign_strict }
      else { math-utils-wasm.mpq_sign }
    )(to-bytes(n))
  )
}

#let /*pub*/ cmp(m, n, strict: false) = {
  let result-byte = (
    if strict { math-utils-wasm.mpq_cmp_strict }
    else { math-utils-wasm.mpq_cmp }
  )(to-bytes(m), to-bytes(n))
  if result-byte.len() == 0 { none }
  else { int.from-bytes(result-byte) }
}

#let /*pub*/ eq(m, n, strict: false) = {
  cmp(m, n, strict: strict) == 0
}