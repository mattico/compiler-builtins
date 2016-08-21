//  When testing functions, QuickCheck (QC) uses small values for integer (`u*`/`i*`) arguments
// (~ `[-100, 100]`), but these values don't stress all the code paths in our intrinsics. Here we
// create newtypes over the primitive integer types with the goal of having full control over the
// random values that will be used to test our intrinsics.

use std::boxed::Box;
use std::fmt;
use core::{f32, f64};

use quickcheck::{Arbitrary, Gen};

use int::LargeInt;
use float::Float;

// Generates values in the full range of the integer type
macro_rules! arbitrary {
    ($TY:ident : $ty:ident) => {
        #[derive(Clone, Copy)]
        pub struct $TY(pub $ty);

        impl Arbitrary for $TY {
            fn arbitrary<G>(g: &mut G) -> $TY
                where G: Gen
            {
                $TY(g.gen())
            }

            fn shrink(&self) -> Box<Iterator<Item=$TY>> {
                struct Shrinker {
                    x: $ty,
                }

                impl Iterator for Shrinker {
                    type Item = $TY;

                    fn next(&mut self) -> Option<$TY> {
                        self.x /= 2;
                        if self.x == 0 {
                            None
                        } else {
                            Some($TY(self.x))
                        }
                    }
                }

                if self.0 == 0 {
                    ::quickcheck::empty_shrinker()
                } else {
                    Box::new(Shrinker { x: self.0 })
                }
            }
        }

        impl fmt::Debug for $TY {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Debug::fmt(&self.0, f)
            }
        }
    }
}

arbitrary!(I32: i32);
arbitrary!(U32: u32);

// These integers are "too large". If we generate e.g. `u64` values in the full range then there's
// only `1 / 2^32` chance of seeing a value smaller than `2^32` (i.e. whose higher "word" (32-bits)
// is `0`)! But this is an important group of values to tests because we have special code paths for
// them. Instead we'll generate e.g. `u64` integers this way: uniformly pick between (a) setting the
// low word to 0 and generating a random high word, (b) vice versa: high word to 0 and random low
// word or (c) generate both words randomly. This let's cover better the code paths in our
// intrinsics.
macro_rules! arbitrary_large {
    ($TY:ident : $ty:ident) => {
        #[derive(Clone, Copy)]
        pub struct $TY(pub $ty);

        impl Arbitrary for $TY {
            fn arbitrary<G>(g: &mut G) -> $TY
                where G: Gen
            {
                if g.gen() {
                    $TY($ty::from_parts(g.gen(), g.gen()))
                } else if g.gen() {
                    $TY($ty::from_parts(0, g.gen()))
                } else {
                    $TY($ty::from_parts(g.gen(), 0))
                }
            }

            fn shrink(&self) -> Box<Iterator<Item=$TY>> {
                struct Shrinker {
                    x: $ty,
                }

                impl Iterator for Shrinker {
                    type Item = $TY;

                    fn next(&mut self) -> Option<$TY> {
                        self.x /= 2;
                        if self.x == 0 {
                            None
                        } else {
                            Some($TY(self.x))
                        }
                    }
                }

                if self.0 == 0 {
                    ::quickcheck::empty_shrinker()
                } else {
                    Box::new(Shrinker { x: self.0 })
                }
            }
        }

        impl fmt::Debug for $TY {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Debug::fmt(&self.0, f)
            }
        }
    }
}

arbitrary_large!(I64: i64);
arbitrary_large!(U64: u64);

macro_rules! arbitrary_float {
    ($TY:ident : $ty:ident) => {
        #[derive(Clone, Copy)]
        pub struct $TY(pub $ty);

        impl Arbitrary for $TY {
            fn arbitrary<G>(g: &mut G) -> $TY
                where G: Gen
            {
                let special = [
                    -0.0, 0.0, $ty::NAN, $ty::INFINITY, -$ty::INFINITY
                ];

                if g.gen_weighted_bool(10) { // Random special case
                    $TY(*g.choose(&special).unwrap())
                } else if g.gen_weighted_bool(10) { // NaN variants
                    let sign: bool = g.gen();
                    let exponent: <$ty as Float>::Int = g.gen();
                    let significand: <$ty as Float>::Int = 0;
                    $TY($ty::from_parts(sign, exponent, significand))
                } else if g.gen() { // Denormalized
                    let sign: bool = g.gen();
                    let exponent: <$ty as Float>::Int = 0;
                    let significand: <$ty as Float>::Int = g.gen();
                    $TY($ty::from_parts(sign, exponent, significand))
                } else { // Random anything
                    let sign: bool = g.gen();
                    let exponent: <$ty as Float>::Int = g.gen();
                    let significand: <$ty as Float>::Int = g.gen();
                    $TY($ty::from_parts(sign, exponent, significand))
                }
            }

            fn shrink(&self) -> Box<Iterator<Item=$TY>> {
                ::quickcheck::empty_shrinker()
            }
        }

        impl fmt::Debug for $TY {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Debug::fmt(&self.0, f)
            }
        }
    }
}

arbitrary_float!(F32: f32);
arbitrary_float!(F64: f64);
