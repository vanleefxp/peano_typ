#import "@preview/lilaq:0.5.0" as lq
#import "../src/lib.typ" as peano /*replace*/
#import peano.func.special: gamma

#set page(width: 48em, height: auto, margin: 2em)

#let mark(mark-func, ..args) = {
  let mark-func = if type(mark-func) == str {
    lq.marks.at(mark-func)
  } else {
    mark-func
  }
  mark-params => {
    let mark-params = mark-params + args.named()
    mark-func(mark-params)
  }
}

#{
  let n-range = (0, 21)

  let n-ball-volume(n) = {
    import calc: pow, pi
    pow(pi, n/2) / gamma(n/2 + 1)
  }
  let n-ball-surface-area(n) = {
    import calc: pow, pi
    2 * pow(pi, n/2) / gamma(n/2)
  }

  let n = range(..n-range)
  grid(
    lq.diagram(
      title: [Volumn of $n$-ball],
      lq.plot(
          n, n.map(n-ball-volume),
          smooth: true,
          stroke: 0.75pt + red,
          mark: mark(".", fill: red, stroke: red),
      ),
    ),
    lq.diagram(
      title: [Surface area of $n$-ball],
      lq.plot(
        n, n.map(n-ball-surface-area),
        smooth: true,
        stroke: 0.75pt + blue,
        mark: mark(".", fill: blue, stroke: blue)
      ),
    ),
    $ V_n = pi^(n"/"2) / Gamma(n"/"2 + 1) $,
    $ S_n = (2 pi ^(n"/"2)) / Gamma(n"/"2) $,
    columns: (1fr,) * 2,
    align: center,
    row-gutter: 0.75em,
  )
}