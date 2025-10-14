# Typst package: `{{name}}`

`{{name}}` is a math utility package that provides you with representations of specialized number types, mathematic special functions, number theory related operations and so on. The name of the package comes from [Peano axioms](https://en.wikipedia.org/wiki/Peano_axioms), which builds up the framework of [natural numbers](https://en.wikipedia.org/wiki/Natural_number) &#x2115;, one of the the most elementary concepts in mathematics, aiming to convey this package's orientation as a simple utility package.

## Number types

`{{name}}` currently supports two number types and their arithmetics:

- `rational`: representation of [rational numbers](https://en.wikipedia.org/wiki/Rational_number) &#x211a; in the form of fractions
- `complex`: representation of[ complex numbers](https://en.wikipedia.org/wiki/Complex_number) &#x2102;

It is a pity that Typst doesn't currently support custom types and overloading operators, so actually these numbers are represented by Typst's `dictionary` type, and you have to invoke specialized methods in the corresponding sub-module to perform arithmetic operations over these numbers.

To use these number types you have to first import the corresponding sub-module:

```typ
#import "@preview/{{name}}:{{version}}"
#import {{name}}.number: rational as q, complex as c
```

Each sub-module contains a method called `from`, by which you can directly create a number instance from a string or a built-in Typst number type. Arithmetic methods in these modules will automatically convert all parameters to the expected number type by this `from` method, so you can simply input strings and built-in number types as parameters.

### Rational numbers

```typ
#import "@preview/{{name}}:{{version}}"
#import {{name}}.number: rational as q

#q.from("1/2") // from string
#q.from(2, 3) // from numerator and denominator
#q.add("1/2", "1/3", "-1/5") // addition
#q.sub("2/3", "1/4") // subtraction
#q.mul("3/4", "2/3", "4/5") // multiplication
#q.div("5/6", "3/2") // division
#q.limit-den(calc.pi, 10000) // limiting maximum denominator
#q.pow("3/2", 5) // raising to an integer power

#q.to-str(q.from(113, 355)) // convert to string
#q.to-math(q.from(113, 355)) // convert to formatted `math.equation` element
```

Currently, `rational.from` supports fraction notation and decimal notation with an optional set of repeating digits enclosed in square brackets.

```typ
#q.from("1/2")
#q.from("-2/3") // sign before numerator
#q.form("5/-4") // sign before denominator
#q.from("1/0")  // infinity
#q.from("-1/0") // negative infinity
#q.from("0/0")  // NaN
```

### Complex numbers

```typ
#import "@preview/{{name}}:{{version}}"
#import {{name}}.number: complex as c

#c.from("1+2i") // from string
#c.from(3, 4) // from real and imaginary parts
#c.add("1+2i", "3+4i", "-2+3i", "6-5i", "2i")
```

## Number theory

The number theory sub module `{{name}}.ntheory` currently supports prime factorization and the extended Euclidean algorithm that gives out the coefficients $u$ and $v$ in Bézout's identity $\gcd (a, b) = u a + v b$.

## Special functions

Special functions such as the gamma function, zeta function, Gauss error function are too specific to mathematics that they are not included in Typst's built-in `calc` module. The `{{name}}.special` sub-module covers these functions. For functions that can be defined in the complex field &#x2102;, these functions support input with `{{name}}`'s complex number type.