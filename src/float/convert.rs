use core::mem;

use float::Float;

/// Implements signed integer to float conversion
/// using the IEEE-754 default round-to-nearest, ties-to-even mode.
#[cfg_attr(not(test), no_mangle)]
pub extern "C" fn __floatsisf(a: i32) -> f32 {
    // Handle zero as a special case
    if a == 0 {
        return <f32>::from_repr(0);
    }

    let sign_bit = <f32>::sign_mask();

    // All other cases begin by extracting the sign and absolute value of a
    let mut a = a;
    let mut sign: <f32 as Float>::Int = 0;
    if a < 0 {
        sign = sign_bit;
        a = a.wrapping_neg();
    };

    // Exponent of (f32)a is the width of abs(a).
    let type_width = mem::size_of::<i32>() as <f32 as Float>::Int * 8;
    let exponent = (type_width - 1).wrapping_sub(a.leading_zeros()) as <f32 as Float>::Int;
    let significand_bits = <f32>::significand_bits();
    let implicit_bit = <f32>::implicit_bit();
    let exponent_bias = <f32>::exponent_bias();
    let mut result: <f32 as Float>::Int;

    // Shift a into the significand field, rounding if it is a right-shift
    if exponent <= significand_bits {
        let shift = significand_bits - exponent;
        result = (a as <f32 as Float>::Int) << shift ^ implicit_bit;
    } else {
        let shift = exponent - significand_bits;
        result = (a as <f32 as Float>::Int) >> shift ^ implicit_bit;
        let round = (a as <f32 as Float>::Int).wrapping_shl(type_width.wrapping_sub(shift));
        if round > sign_bit { result += 1; }
        if round == sign_bit { result += result & 1; }
    }

    // Insert the exponent
    result = result.wrapping_add(((exponent + exponent_bias) as <f32 as Float>::Int) << significand_bits);

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
