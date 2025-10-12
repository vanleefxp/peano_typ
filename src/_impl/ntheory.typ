#let math-utils-wasm = plugin("math-utils.wasm")

#let prime-fac(num) = {
  cbor(math-utils-wasm.prime_factors(num.to-bytes(endian: "little")))
}

#let egcd(a, b) = {
  cbor(
    math-utils-wasm.extended_gcd(
      a.to-bytes(endian: "little"),
      b.to-bytes(endian: "little"),
    )
  )
}
