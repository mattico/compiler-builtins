use core::mem;
use core::num::Wrapping;

use float::Float;

// TODO:
// - [ ] floatdidf
// - [ ] floatdisf
// - [ ] floatsidf
// - [x] floatsisf
// - [ ] floatundidf
// - [ ] floatundisf
// - [ ] floatunsidf
// - [ ] floatunsisf

// FIXME(rust#23545) a few of these casts are unnecessary
// if Wrapping actually supported all of the shifts.

/// Implements signed integer to float conversion
/// using the IEEE-754 default round-to-nearest, ties-to-even mode.
#[cfg_attr(not(test), no_mangle)]
pub extern "C" fn __floatsisf(a: i32) -> f32 {
    // Handle zero as a special case
    if a == 0 {
        return <f32>::from_repr(<f32>::zero());
    }

    let sign_bit = <f32>::sign_bit();

    // All other cases begin by extracting the sign and absolute value of a
    let mut a = a;
    let mut sign = <f32 as Float>::zero();
    if a < 0 {
        sign = sign_bit;
        a = a.wrapping_neg();
    };

    // Exponent of (f32)a is the width of abs(a).
    let type_width = Wrapping(mem::size_of::<i32>() as <f32 as Float>::Int * 8);
    let exponent = (type_width - <f32>::one()) - Wrapping(a.leading_zeros());
    let significand_bits = <f32>::significand_bits();
    let implicit_bit = <f32>::implicit_bit();
    let exponent_bias = <f32>::exponent_bias();
    let mut result = <f32 as Float>::zero();

    let a = Wrapping(a as <f32 as Float>::Int);

    // Shift a into the significand field, rounding if it is a right-shift
    if exponent <= significand_bits {
        let shift = significand_bits - exponent;
        result = a << shift.0 as usize ^ implicit_bit;
    } else {
        let shift = exponent - significand_bits;
        result = a >> shift.0 as usize ^ implicit_bit;
        let round = a << (type_width - shift).0 as usize;
        if round > sign_bit { result += <f32>::one(); }
        if round == sign_bit { result += result & <f32>::one(); }
    }

    // Insert the exponent
    result = result + ((exponent + exponent_bias) << significand_bits.0 as usize);

    // Insert the sign bit and return
    <f32>::from_repr(result | sign)
}

#[cfg(test)]
mod tests {
    use qc::{I32, F32};

    check! {
        fn __floatsisf(f: extern fn(i32) -> f32, a: I32) -> Option<F32> {
            Some(F32(f(a.0)))
        }
    }
}
