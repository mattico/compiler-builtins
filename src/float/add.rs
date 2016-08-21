use core::{f32, f64};
use core::num::{Float as f, FpCategory};
use float::Float;

macro_rules! add {
    ($intrinsic:ident: $ty:ident) => {
        /// Returns `a + b`
        #[allow(unused_parens)]
        #[cfg_attr(not(test), no_mangle)]
        pub extern fn $intrinsic(a: $ty, b: $ty) -> $ty {
            let bits =              <$ty>::bits() as <$ty as Float>::Int;
            let significand_bits =  <$ty>::significand_bits() as <$ty as Float>::Int;
            let exponent_bits =     <$ty>::exponent_bits() as <$ty as Float>::Int;
            let max_exponent =      (1 << exponent_bits as usize) - 1;

            let implicit_bit =      1 << significand_bits as usize;
            let significand_mask =  implicit_bit - 1;
            let sign_bit =          1 << (significand_bits + exponent_bits) as usize;
            let abs_mask =          sign_bit - 1;
            let exponent_mask =     abs_mask ^ significand_mask;
            let inf_rep =           exponent_mask;
            let quiet_bit =         implicit_bit >> 1;

            let a_rep =         a.repr();
            let b_rep =         b.repr();
            let a_abs =             a_rep & abs_mask;
            let b_abs =             b_rep & abs_mask;

            // Classify each input
            match ($ty::classify(a), $ty::classify(b)) {
                (FpCategory::Nan, _) => { // NaN + anything = qNaN
                    return (<$ty as Float>::from_repr((a_abs | quiet_bit)));
                },
                (_, FpCategory::Nan) => { // anything + NaN = qNaN
                    return (<$ty as Float>::from_repr((b_abs | quiet_bit)));
                },
                (FpCategory::Infinite, FpCategory::Infinite) => { 
                    if a.sign() != b.sign() {
                        // +/-infinity + -/+infinity = qNaN
                        return ($ty::NAN);
                    } else {
                        // +/-infinity + anything remaining = +/- infinity
                        return a;
                    }
                },
                (_, FpCategory::Infinite) => {
                    // +/-infinity + anything remaining = +/- infinity
                    return b;
                },
                (FpCategory::Zero, _) => {
                    // zero + anything = anything
                    if a.sign() != b.sign() {
                        // but we need to get the sign right for zero + zero
                        return (<$ty as Float>::from_repr(a.repr() & b.repr()));
                    } else {
                        return b;
                    }
                },
                (_, FpCategory::Zero) => {
                    // anything + zero = anything
                    return a;
                },
                (cat_a @ _, cat_b @ _) => {
                    let (mut a_exponent, mut a_significand) = match cat_a {
                        FpCategory::Subnormal => <$ty>::normalize(a.exponent()),
                        FpCategory::Normal => (a.exponent() as i32, a.significand()),
                        _ => unreachable!(),
                    };
                    let (mut b_exponent, mut b_significand) = match cat_b {
                        FpCategory::Subnormal => <$ty>::normalize(b.exponent()),
                        FpCategory::Normal => (b.exponent() as i32, b.significand()),
                        _ => unreachable!(),
                    };

                    // Make sure that `a` is the larger argument
                    if b_abs > a_abs {
                        ::core::mem::swap(&mut a_exponent, &mut b_exponent);
                        ::core::mem::swap(&mut a_significand, &mut b_significand);
                    }

                    // The sign of the result is the sign of the larger operand, a.  If they
                    // have opposite signs, we are performing a subtraction; otherwise addition.
                    let result_sign = (a.sign() as <$ty as Float>::Int).wrapping_shl(bits as u32);
                    let subtraction = a.sign() != b.sign();

                    // Shift the significands to give us round, guard and sticky, and or in the
                    // implicit significand bit.  (If we fell through from the denormal path it
                    // was already set by normalize(), but setting it twice won't hurt
                    // anything.)
                    a_significand = (a_significand | implicit_bit) << 3;
                    b_significand = (b_significand | implicit_bit) << 3;

                    // Shift the significand of b by the difference in exponents, with a sticky
                    // bottom bit to get rounding correct.
                    let align = a_exponent.wrapping_sub(b_exponent);
                    if align != 0 {
                        if align < bits as i32 {
                            let sticky = (b_significand.wrapping_shl((bits as i32).wrapping_sub(align) as u32) != 0) as <$ty as Float>::Int;
                            b_significand = b_significand.wrapping_shr(align as u32) | sticky;
                        } else {
                            b_significand = 1; // sticky; b is known to be non-zero.
                        }
                    }
                    if subtraction {
                        a_significand = a_significand.wrapping_sub(b_significand);
                        // If a == -b, return +zero.
                        if a_significand == 0 {
                            return 0.0;
                        }

                        // If partial cancellation occured, we need to left-shift the result
                        // and adjust the exponent:
                        if a_significand < implicit_bit << 3 {
                            let shift = (a_significand.leading_zeros() as i32)
                                .wrapping_sub((implicit_bit << 3).leading_zeros() as i32);
                            a_significand = a_significand.wrapping_shl(shift as u32);
                            a_exponent = a_exponent.wrapping_sub(shift);
                        }
                    } else /* addition */ {
                        a_significand = a_significand.wrapping_add(b_significand);

                        // If the addition carried up, we need to right-shift the result and
                        // adjust the exponent:
                        if (a_significand & implicit_bit << 4) != 0 {
                            let sticky = ((a_significand & 1) != 0) as <$ty as Float>::Int;
                            a_significand = a_significand.wrapping_shr(1) | sticky;
                            a_exponent = a_exponent.wrapping_add(1);
                        }
                    }

                    // If we have overflowed the type, return +/- infinity:
                    if a_exponent >= max_exponent as i32 {
                        return (<$ty>::from_repr(inf_rep | result_sign as <$ty as Float>::Int));
                    }

                    if a_exponent <= 0 {
                        // Result is denormal before rounding; the exponent is zero and we
                        // need to shift the significand.
                        let shift = (1i32).wrapping_sub(a_exponent) as <$ty as Float>::Int;
                        let sticky = (a_significand.wrapping_shl(
                            bits.wrapping_sub(shift as <$ty as Float>::Int) as u32) != 0) as <$ty as Float>::Int;
                        a_significand = a_significand.wrapping_shr(shift as u32) | sticky;
                        a_exponent = 0;
                    }

                    // Low three bits are round, guard, and sticky.
                    let round_guard_sticky = (a_significand & 0x7) as i32;

                    // Shift the significand into place, and mask off the implicit bit.
                    let mut result = a_significand.wrapping_shr(3) & significand_mask;

                    // Insert the exponent and sign.
                    result |= a_exponent.wrapping_shl(significand_bits as u32) as <$ty as Float>::Int;
                    result |= (result_sign as <$ty as Float>::Int).wrapping_shl(bits as u32);

                    // Final rounding.  The result may overflow to infinity, but that is the
                    // correct result in that case.
                    if round_guard_sticky > 0x4 { result = result.wrapping_add(1); }
                    if round_guard_sticky == 0x4 { result = result.wrapping_add(result & 1); }
                    
                    return (<$ty>::from_repr(result));
                }
            }
        }
    }
}

add!(__addsf3: f32);
add!(__adddf3: f64);

// FIXME: Implement these using aliases
#[cfg(target_arch = "arm")]
#[cfg_attr(not(test), no_mangle)]
pub extern fn __aeabi_dadd(a: f64, b: f64) -> f64 {
    __adddf3(a, b)
}

#[cfg(target_arch = "arm")]
#[cfg_attr(not(test), no_mangle)]
pub extern fn __aeabi_fadd(a: f32, b: f32) -> f32 {
    __addsf3(a, b)
}

#[cfg(test)]
mod tests {
    use float::Float;
    use qc::{F32, F64};
    use test::{black_box, Bencher};

    // NOTE The tests below have special handing for NaN values.
    // Because NaN != NaN, the representations are compared
    // Because there are many diffferent values of NaN, and the implementation
    // doesn't care about calculating the 'correct' one, if both values are NaN
    // the values are considered equivalent.

    quickcheck! {
        fn addsf3(a: F32, b: F32) -> bool {
            let (a, b) = (a.0, b.0);
            let x = super::__addsf3(a, b);
            let y = a + b;
            if !(x.is_nan() && y.is_nan()) {
                x.repr() == y.repr()
            } else {
                true
            }
        }

        fn adddf3(a: F64, b: F64) -> bool {
            let (a, b) = (a.0, b.0);
            let x = super::__adddf3(a, b);
            let y = a + b;
            if !(x.is_nan() && y.is_nan()) {
                x.repr() == y.repr()
            } else {
                true
            }
        }
    }

    #[bench]
    fn bench_native_float_add(b: &mut Bencher) {
        b.iter(|| {
            let n = black_box(1000);
            (0..n).fold(0.0, |a, b| a + b as f32)
        });
    }

    #[bench]
    fn bench_builtin_float_add(b: &mut Bencher) {
        b.iter(|| {
            let n = black_box(1000);
            (0..n).fold(0.0, |a, b| super::__addsf3(a, b as f32))
        });
    }

    #[bench]
    fn bench_native_double_add(b: &mut Bencher) {
        b.iter(|| {
            let n = black_box(1000);
            (0..n).fold(0.0, |a, b| a + b as f64)
        });
    }

    #[bench]
    fn bench_builtin_double_add(b: &mut Bencher) {
        b.iter(|| {
            let n = black_box(1000);
            (0..n).fold(0.0, |a, b| super::__adddf3(a, b as f64))
        });
    }
}
