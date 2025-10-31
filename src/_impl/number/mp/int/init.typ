#import "@preview/elembic:1.1.1" as e
#let math-utils-wasm = plugin("../../../math-utils.wasm")

#let mp-int = e.types.declare(
  "mp-int",
  prefix: "peano.number",
  fields: (
    e.field(
      "buffer",
      bytes,
    ),
  ),
)
#let zero-byte = bytes((0,))

#let mp-int-buffer(src) = {
  if type(src) == int or type(src) == float {
    math-utils-wasm.mpz_from_int(int(src).to-bytes())
  } else if type(src) == decimal {
    let src = str(src)
    if "." in src {
      src = src.split(".").at(0)
    }
    math-utils-wasm.parse_mpz(bytes(src))
  } else if type(src) == str {
    math-utils-wasm.parse_mpz(bytes(src))
  } else {
    panic("unsupported input type: `" + repr(type(src)) + "`.")
  }
}

#let /*pub*/ from(src) = {
  mp-int(buffer: mp-int-buffer(src))
}

#let /*pub*/ is_(obj) = {
  e.tid(obj) == e.tid(mp-int)
}

#let /*pub*/ to-bytes(n) = {
  if is_(n) {
    n.buffer
  } else {
    mp-int-buffer(n)
  }
}

#let verify-bytes(buffer) = {
  math-utils-wasm.verify_mpz(buffer) != zero-byte
}

#let /*pub*/ from-bytes(buffer) = {
  assert(verify-bytes(buffer), "Invalid bytes input for `mp-int`.")
  mp-int(buffer: buffer)
}

#let encode-mpz-seq(obj) = {
  cbor.encode(obj.map(cbor))
}