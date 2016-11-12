#[cfg(not(all(feature = "c", target_arch = "x86")))]
use int::{Int, LargeInt};

macro_rules! ashl {
    ($intrinsic:ident: $ty:ty) => {
        /// Returns `a << b`, requires `b < $ty::bits()`
        #[cfg_attr(not(test), no_mangle)]
        pub extern "aapcs" fn $intrinsic(a: $ty, b: u32) -> $ty {
            let half_bits = <$ty>::bits() / 2;
            if b & half_bits != 0 {
                <$ty>::from_parts(0, a.low() << (b - half_bits))
            } else if b == 0 {
                a
            } else {
                <$ty>::from_parts(a.low() << b, (a.high() << b) | (a.low() >> (half_bits - b)))
            }
        }
    }
}

macro_rules! ashr {
    ($intrinsic:ident: $ty:ty) => {
        /// Returns arithmetic `a >> b`, requires `b < $ty::bits()`
        #[cfg_attr(not(test), no_mangle)]
        pub extern "aapcs" fn $intrinsic(a: $ty, b: u32) -> $ty {
            let half_bits = <$ty>::bits() / 2;
            if b & half_bits != 0 {
                <$ty>::from_parts((a.high() >> (b - half_bits)) as <$ty as LargeInt>::LowHalf,
                                  a.high() >> (half_bits - 1))
            } else if b == 0 {
                a
            } else {
                let high_unsigned = a.high() as <$ty as LargeInt>::LowHalf;
                <$ty>::from_parts((high_unsigned << (half_bits - b)) | (a.low() >> b),
                                  a.high() >> b)
            }
        }
    }
}

macro_rules! lshr {
    ($intrinsic:ident: $ty:ty) => {
        /// Returns logical `a >> b`, requires `b < $ty::bits()`
        #[cfg_attr(not(test), no_mangle)]
        pub extern "aapcs" fn $intrinsic(a: $ty, b: u32) -> $ty {
            let half_bits = <$ty>::bits() / 2;
            if b & half_bits != 0 {
                <$ty>::from_parts(a.high() >> (b - half_bits), 0)
            } else if b == 0 {
                a
            } else {
                <$ty>::from_parts((a.high() << (half_bits - b)) | (a.low() >> b), a.high() >> b)
            }
        }
    }
}

#[cfg(not(all(feature = "c", target_arch = "x86")))]
ashl!(__ashldi3: u64);

#[cfg(not(all(feature = "c", target_arch = "x86")))]
ashr!(__ashrdi3: i64);

#[cfg(not(all(feature = "c", target_arch = "x86")))]
lshr!(__lshrdi3: u64);

#[cfg(test)]
mod tests {
    use qc::{I64, U64};

    // NOTE We purposefully stick to `u32` for `b` here because we want "small" values (b < 64)
    check! {
        fn __ashldi3(f: extern "aapcs" fn(u64, u32) -> u64, a: U64, b: u32) -> Option<u64> {
            let a = a.0;
            if b >= 64 {
                None
            } else {
                Some(f(a, b))
            }
        }

        fn __ashrdi3(f: extern "aapcs" fn(i64, u32) -> i64, a: I64, b: u32) -> Option<i64> {
            let a = a.0;
            if b >= 64 {
                None
            } else {
                Some(f(a, b))
            }
        }

        fn __lshrdi3(f: extern "aapcs" fn(u64, u32) -> u64, a: U64, b: u32) -> Option<u64> {
            let a = a.0;
            if b >= 64 {
                None
            } else {
                Some(f(a, b))
            }
        }
    }
}
