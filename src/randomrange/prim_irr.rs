use rand_xoshiro::rand_core::Rng;
use crate::{DiceExt, DiceExtSides, Rng128Ext};

use super::InclusiveRandomRange;

#[cfg(feature = "f128-stable")]
impl InclusiveRandomRange<f128> for std::ops::RangeInclusive<f64> {
    fn random_of(&self) -> f128 {
        let (mut start, mut end) = (*self.start(), *self.end());
        if start > end { std::mem::swap(&mut start, &mut end); }// swap endpoints if needed…

        let raw_bits = engine::GLOBAL_REACTOR_U128.roll(u128::MAX);// 128-bit reactor needed
        let max_m = (1u128 << 113) - 1; // use 113 bits of mantissa of the f128
        start + ((raw_bits & max_m) as f128 / max_m as f128) * (end - start)
    }
}

impl InclusiveRandomRange<f64> for std::ops::RangeInclusive<f64> {
    fn random_of(&self) -> f64 {
        let (mut start, mut end) = (*self.start(), *self.end());
        if start > end { std::mem::swap(&mut start, &mut end); }// swap endpoints if needed…

        let raw_bits = 1.d(u64::MAX as DiceExtSides);
        let max_m = (1u64 << 53) - 1; // use 53 bits of mantissa of the f64
        start + ((raw_bits & max_m) as f64 / max_m as f64) * (end - start)
    }
}

impl InclusiveRandomRange<f32> for std::ops::RangeInclusive<f32> {
    fn random_of(&self) -> f32 {
        let (mut start, mut end) = (*self.start(), *self.end());
        if start > end { std::mem::swap(&mut start, &mut end); }// swap endpoints if needed…

        let raw_bits = 1.d(u64::MAX as DiceExtSides);
        let max_m = (1u32 << 24) - 1; // use 24 bits of mantissa of the f32
        start + ((raw_bits as u32 & max_m) as f32 / max_m as f32) * (end - start)
    }
}

impl InclusiveRandomRange<char> for std::ops::RangeInclusive<char> {
    fn random_of(&self) -> char {
        let (mut start, mut end) = (*self.start(), *self.end());
        if start > end { std::mem::swap(&mut start, &mut end); }// swap endpoints if needed…
        
        let u_start = start as u32;
        let u_end = end as u32;
        let range = u_end - u_start + 1;
        loop {
            let offt = 1.d(range as u64 as DiceExtSides) as u32;
            let maybe = u_start + offt;
            // skip the illegal surrogate gap
            if !(0xD800..=0xDFFF).contains(&maybe) {
                return unsafe { char::from_u32_unchecked(maybe) }
            }
        }
    }
}

macro_rules! impl_inclusive_rr {
    ([$($t:tt),*]) => { paste::paste! {$(
        impl_inclusive_rr!(engine $t u [<u $t>]);
        impl_inclusive_rr!(engine $t i [<i $t>]);
    )+}};

    (core $bits:tt $self:tt, $core_type:ty, $t:ty) => { paste::paste! {{
        let (mut start, mut end) = (*$self.start(), *$self.end());
        // in case someone fed a range like 12..=6 ... play along and just inverse the ends.
        if start > end {
            std::mem::swap(&mut start, &mut end);
            log::warn!("I'd suggest reversal of start/end of the range, but playing along for now and inversing the ends myself.");
        }

        let start = start as $core_type;
        let end = end as $core_type;
        // full domain roll?
        if (start as $t) == <$t>::MIN && (end as $t) == <$t>::MAX {
            return crate::chaos_dice().rng().[<next_u $bits>]() as $t
        }
        let sides = end - start + 1;
        // engine.roll(X) gives 1..=X value. Shift it down.
        (start + (1.d(sides as u64 as DiceExtSides) - 1) as $core_type) as $t
    }}};

    (engine 128 $sign:tt $t:ty) => { impl_inclusive_rr!(128 $sign $t); };
    (engine $_:tt $sign:tt $t:ty) => { impl_inclusive_rr!(64 $sign $t); };
    ($engine_bits:tt $sign:tt $t:ty) => { paste::paste! {
        impl InclusiveRandomRange<$t> for std::ops::RangeInclusive<$t> {
            /// Generate random value within the given range.
            /// 
            /// ```
            /// use dicebag::InclusiveRandomRange;
            /// use std::ops::RangeInclusive as RI;
            /// let range: RI = 6..=12;
            /// let roll = range.random_of();
            /// ```
            fn random_of(&self) -> $t {
                impl_inclusive_rr!(core $engine_bits self, [<$sign 128>], $t) as $t
            }
        }
    }};
}
impl_inclusive_rr!([8,16,32,64,128,size]);

#[cfg(test)]
mod prim_irr_tests {
    use crate::InclusiveRandomRange;

    #[test]
    fn irr_128bit_coverage() {
        let r = i128::MIN..=i128::MAX;
        let x = r.random_of();
        assert!(x >= i128::MIN && x <= i128::MAX);
    }

    #[test]
    fn irr_8bit_coverage() {
        let r = i8::MIN..=i8::MAX;
        let x = r.random_of();
        assert!(x >= i8::MIN && x <= i8::MAX);
    }
}
