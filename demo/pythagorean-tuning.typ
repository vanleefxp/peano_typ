#import "@preview/cetz:0.4.2"
#import "../src/lib.typ" as peano /*replace*/
#import peano.number: q

#set page(width: auto, height: auto, margin: 2em)

#{
  let dot = circle(radius: 1pt, fill: black)
  let log2 = calc.log.with(base: 2)

  let freqs = ()
  let freq = q.from("3/2")
  for i in range(11) {
    freqs.push(freq)
    freq = q.mul(freq, "3/2")
    if q.cmp(freq, 2) > 0 {
      freq = q.div(freq, 2)
    }
  }

  cetz.canvas(
    {
      import cetz.draw: *
      set-style(
        line: (stroke: 0.5pt),
      )
      line((0, 0), (1, 0))
      content((0, 0), dot)
      content((1, 0), dot)
      content((0, 0), pad($1$, right: 3pt), anchor: "east")
      content((1, 0), pad($2$, left: 3pt), anchor: "west")
      for (i, freq) in freqs.enumerate() {
        let x = log2(q.to-float(freq))
        let anchor
        let padding
        if i < 7 {
          anchor = "south"
          padding = (bottom: 3pt)
        } else {
          anchor = "north"
          padding = (top: 5pt)
        }
        content((x, 0), dot)
        content(
          (x, 0),
          pad(q.to-math(freq), ..padding),
          anchor: anchor,
        )
      }
    },
    length: 24em
  )
}