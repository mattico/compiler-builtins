use core::mem;

use float::Float;

//ARM_EABI_FNALIAS(i2f, floatsisf)

/// Macro for implementing signed integer to float conversion
/// using the IEEE-754 default round-to-nearest, ties-to-even mode.
macro_rules! int_to_float {
    ($intrinsic:ident: $ty:ty => $fty:ty) => {
        pub extern fn $intrinsic(a: $ty) -> $fty {
    
            // Handle zero as a special case to protect clz
            // if (a == 0)
            //     return fromRep(0);
            if a == 0 {
                return <$fty>::from_repr(0);
            }

            // Exponent of (fp_t)a is the width of abs(a).
            // const int exponent = (aWidth - 1) - __builtin_clz(a);
            // rep_t result;
            let type_width = mem::size_of::<$ty>() as <$fty as Float>::Int;
            let exponent = (type_width - 1).wrapping_sub(a.leading_zeros()) as <$fty as Float>::Int;
            let significand_bits = <$fty>::significand_bits();
            let sign_bit = <$fty>::sign_mask();
            let implicit_bit = <$fty>::implicit_bit();
            let exponent_bias = <$fty>::exponent_bias();
            let mut result: <$fty as Float>::Int;
    
            // All other cases begin by extracting the sign and absolute value of a
            // rep_t sign = 0;
            // if (a < 0) {
            //     sign = signBit;
            //     a = -a;
            // }
            let (sign, a_rep) = if a < 0 {
                (sign_bit, -a as <$fty as Float>::Int)
            } else {
                (0, a as <$fty as Float>::Int)
            };
    
            // Shift a into the significand field, rounding if it is a right-shift
            // if (exponent <= significandBits) {
            //     const int shift = significandBits - exponent;
            //     result = (rep_t)a << shift ^ implicitBit;
            // } else {
            //     const int shift = exponent - significandBits;
            //     result = (rep_t)a >> shift ^ implicitBit;
            //     rep_t round = (rep_t)a << (typeWidth - shift);
            //     if (round > signBit) result++;
            //     if (round == signBit) result += result & 1;
            // }
            if exponent <= significand_bits {
                let shift = significand_bits - exponent;
                result = a_rep << shift ^ implicit_bit;
            } else {
                let shift = exponent - significand_bits;
                result = a_rep.wrapping_shr(shift ^ implicit_bit);
                let round = a_rep.wrapping_shl(type_width.wrapping_sub(shift));
                if round > sign_bit { result += 1; }
                if round == sign_bit { result += result & 1; }
            }
    
            // Insert the exponent
            // result += (rep_t)(exponent + exponentBias) << significandBits;
            // Insert the sign bit and return
            // return fromRep(result | sign);
            result = result.wrapping_add(((exponent + exponent_bias) as <$fty as Float>::Int) << significand_bits);
            
            <$fty>::from_repr(result | sign)
        }
    }
}

int_to_float!(__floatsisf: i32 => f32);

#[cfg(test)]
mod tests {
    use qc::{I32, F32};

    check! {
        fn __floatsisf(f: extern fn(i32) -> f32, a: I32) -> Option<F32> {
            Some(F32(f(a.0)))
        }
    }
}
