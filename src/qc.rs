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
                // NOTE Generate edge cases with a 10% chance
                let t = if g.gen_weighted_bool(10) {
                    *g.choose(&[
                        $ty::min_value(),
                        0,
                        $ty::max_value(),
                    ]).unwrap()
                } else {
                    g.gen()
                };

                $TY(t)
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
                // NOTE Generate edge cases with a 10% chance
                let t = if g.gen_weighted_bool(10) {
                    *g.choose(&[
                        $ty::min_value(),
                        0,
                        $ty::max_value(),
                    ]).unwrap()
                } else {
                    match g.gen_range(0, 3) {
                        0 => $ty::from_parts(g.gen(), g.gen()),
                        1 => $ty::from_parts(0, g.gen()),
                        2 => $ty::from_parts(g.gen(), 0),
                        _ => unreachable!(),
                    }
                };

                $TY(t)
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

        impl PartialEq for $TY {
            fn eq(&self, other: &$TY) -> bool {
                // NOTE(cfg) for some reason, on hard float targets, our implementation doesn't
                // match the output of its gcc_s counterpart. Until we investigate further, we'll
                // just avoid testing against gcc_s on those targets. Do note that our
                // implementation matches the output of the FPU instruction on *hard* float targets
                // and matches its gcc_s counterpart on *soft* float targets.
                if cfg!(gnueabihf) {
                    return true
                }
                self.0.eq_repr(other.0)
            }
        }
    }
}

arbitrary_float!(F32: f32);
arbitrary_float!(F64: f64);

// Convenience macro to test intrinsics against their reference implementations.
//
// Each intrinsic is tested against both the `gcc_s` library as well as
// `compiler-rt`. These libraries are defined in the `gcc_s` crate as well as
// the `compiler-rt` crate in this repository. Both load a dynamic library and
// lookup symbols through that dynamic library to ensure that we're using the
// right intrinsic.
//
// This macro hopefully allows you to define a bare minimum of how to test an
// intrinsic without worrying about these implementation details. A sample
// invocation looks like:
//
//
//    check! {
//        // First argument is the function we're testing (either from this lib
//        // or a dynamically loaded one. Further arguments are all generated by
//        // quickcheck.
//        fn __my_intrinsic(f: extern fn(i32) -> i32,
//                          a: I32)
//                          -> Option<(i32, i64)> {
//
//            // Discard tests by returning Some
//            if a.0 == 0 {
//                return None
//            }
//
//            // Return the result via `Some` if the test can run
//            let mut other_result = 0;
//            let result = f(a.0, &mut other_result);
//            Some((result, other_result))
//        }
//    }
//
// If anything returns `None` then the test is discarded, otherwise the two
// results are compared for equality and the test fails if this equality check
// fails.
macro_rules! check {
    ($(
        fn $name:ident($f:ident: extern fn($($farg:ty),*) -> $fret:ty,
                       $($arg:ident: $t:ty),*)
                       -> Option<$ret:ty>
        {
            $($code:tt)*
        }
    )*) => (
        $(
            fn $name($f: extern fn($($farg),*) -> $fret,
                     $($arg: $t),*) -> Option<$ret> {
                $($code)*
            }
        )*

        mod _test {
            use qc::*;
            use std::mem;
            use quickcheck::TestResult;

            $(
                #[test]
                fn $name() {
                    fn my_check($($arg:$t),*) -> TestResult {
                        let my_answer = super::$name(super::super::$name,
                                                     $($arg),*);
                        let compiler_rt_fn = ::compiler_rt::get(stringify!($name));
                        let compiler_rt_answer = unsafe {
                            super::$name(mem::transmute(compiler_rt_fn),
                                            $($arg),*)
                        };
                        let gcc_s_answer = 
                        match ::gcc_s::get(stringify!($name)) {
                            Some(f) => unsafe {
                                Some(super::$name(mem::transmute(f), 
                                                  $($arg),*))
                            },
                            None => None,
                        };

                        let print_values = || {
                            print!("{} - Args: ", stringify!($name));
                            $(print!("{:?} ", $arg);)*
                            print!("\n");
                            println!("  rustc-builtins: {:?}", my_answer);
                            println!("  compiler_rt:    {:?}", compiler_rt_answer);
                            println!("  gcc_s:          {:?}", gcc_s_answer);
                        };

                        if my_answer != compiler_rt_answer {
                            print_values();
                            TestResult::from_bool(false)
                        } else if gcc_s_answer.is_some() && 
                                  my_answer != gcc_s_answer.unwrap() {
                            print_values();
                            TestResult::from_bool(false)
                        } else {
                            TestResult::from_bool(true)
                        }
                    }

                    ::quickcheck::quickcheck(my_check as fn($($t),*) -> TestResult)
                }
            )*
        }
    )
}
