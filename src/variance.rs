use num::Float;

/// Percentage amount value variator(s).
pub trait PercentageVariance {
    /// Take a number and alter it by up to (or less, of course) ±X%.
    fn jitter_percentage(&self, percentage: f64) -> Self;
}

/// Fixed value value variator(s).
pub trait FixedNumberVariance<T: Float> {
    /// Take a number and alter it ± by \[**0 .. *upto***\].
    fn jitter_within(&self, upto: T) -> T;
}

/// Implement some dice extensions for float types.
macro_rules! implement_float_variance {
    ([$($t:ty),*]) => { $( implement_float_variance!($t); )+ };
    // (f128) => { implement_float_variance!(reactor U128, u128, 113, f128); };
    ($t:ty) => { implement_float_variance!(reactor U64, u64, 63, $t); };
    (reactor $r:ident, $type:ty, $mantissa_bits:literal, $t:ty) => {paste::paste!{
        impl FixedNumberVariance<$t> for $t {
            fn jitter_within(&self, upto: Self) -> Self {
                use super::DiceExt;
                if upto <= 0.0 { return *self; }
                let raw_bits = 1_u64.d($type::MAX as usize);
                let mantissa_bits = ([<$t>]::MANTISSA_DIGITS as u32).min($mantissa_bits);
                let max_mask = ((1 as $type) << mantissa_bits) - 1;
                let scale = (raw_bits & max_mask) as $t / max_mask as $t;
                self + ((scale * 2.0 * upto ) - upto)
            }
        }

        impl PercentageVariance for $t {
            fn jitter_percentage(&self, percentage: f64) -> Self {
                let p = 0.01 * percentage as $t;
                self.jitter_within( self.abs() * p )
            }
        }
    }};
}
implement_float_variance!([f32,f64]);
#[cfg(feature = "f128-stable")]
implement_float_variance!(f128);
