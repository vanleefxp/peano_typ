// -> number/complex/repr.typ

#import "@preview/oxifmt:1.0.0": strfmt
#import "init.typ": complex

#let /*pub*/ to-str(z) = {
  let z = complex(z)
  strfmt("{re:.2}{im:+.2}i", re: z.re, im: z.im)
}