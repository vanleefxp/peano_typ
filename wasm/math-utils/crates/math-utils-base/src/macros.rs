#[macro_export]
macro_rules! impl_sum {
    ($t:ty) => {
        impl Sum for $t {
            fn sum<I>(xs: I) -> $t
            where
                I: Iterator<Item = $t>,
            {
                let mut stack = Vec::new();
                for (i, x) in xs.enumerate() {
                    if x.is_nan() {
                        return <$t>::NAN;
                    }
                    let mut s = x;
                    for _ in 0..(i + 1).trailing_zeros() {
                        s += stack.pop().unwrap();
                    }
                    stack.push(s);
                }
                let mut s = <$t>::ZERO;
                for x in stack.into_iter().rev() {
                    s += x;
                }
                s
            }
        }

        impl<'a> Sum<&'a $t> for $t {
            fn sum<I>(xs: I) -> $t
            where
                I: Iterator<Item = &'a $t>,
            {
                let mut stack = Vec::new();
                for (i, x) in xs.enumerate() {
                    if x.is_nan() {
                        return <$t>::NAN;
                    }
                    let mut s = x.clone();
                    for _ in 0..(i + 1).trailing_zeros() {
                        s += stack.pop().unwrap();
                    }
                    stack.push(s);
                }
                let mut s = <$t>::ZERO;
                for x in stack.into_iter().rev() {
                    s += x;
                }
                s
            }
        }
    };
}

#[macro_export]
macro_rules! impl_product {
    ($t:ty) => {
        impl Product for $t {
            fn product<I>(xs: I) -> $t
            where
                I: Iterator<Item = $t>,
            {
                let mut stack = Vec::new();
                for (i, x) in xs.enumerate() {
                    if x.is_nan() {
                        return <$t>::NAN;
                    }
                    let mut s = x;
                    for _ in 0..(i + 1).trailing_zeros() {
                        s *= stack.pop().unwrap();
                    }
                    stack.push(s);
                }
                let mut s = <$t>::ONE;
                for x in stack.into_iter().rev() {
                    s *= x;
                }
                s
            }
        }

        impl<'a> Product<&'a $t> for $t {
            fn product<I>(xs: I) -> $t
            where
                I: Iterator<Item = &'a $t>,
            {
                let mut stack = Vec::new();
                for (i, x) in xs.enumerate() {
                    if x.is_nan() {
                        return <$t>::NAN;
                    }
                    let mut s = x.clone();
                    for _ in 0..(i + 1).trailing_zeros() {
                        s *= stack.pop().unwrap();
                    }
                    stack.push(s);
                }
                let mut s = <$t>::ONE;
                for x in stack.into_iter().rev() {
                    s *= x;
                }
                s
            }
        }
    };
}
