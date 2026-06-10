//! Dice rolling!
//! 
//! In most cases, anything from `i8`/`u8` up to `i128`/`u128`
//! and `usize` is supported (alongside `f32`/`f64` for a few
//! functions).
//! 
//! # The Extensions
//! 
//! ## `DiceExt`
//! 
//! Covers the core intent dice rollings, e.g. `3.d6()`, `2.d10()`.
//! 
//! ```
//! use dicebag::DiceExt;
//! let a = 3.d6();
//! let b = 2_u8.d4();
//! let c = 5.d(a-3); // FYI: zero as dice size results in 0, no matter the number of dice...
//! ```
//! 
//! ## `HiLo`
//! 
//! Coin flipping, using whatever datatype you implement it for…
//! Comes with two convenience macros so that you don't need to
//! write those yourself:
//! ```
//! use dicebag::{DiceExt, HiLo, lo, hi};
//! if lo!() {/* do something if result was "low" */}
//! if hi!() {/* do something if result was "high" */}
//! ```
//! 
//! ## `InclusiveRandomRange`
//! 
//! To get a random value within given range.
//! ```
//! use dicebag::InclusiveRandomRange;
//! let range = 6..=12;
//! let v = range.random_of();
//! ```
//! 
//! ## `RandomOf<T>`
//! 
//! A trait to get some random entry of e.g. [Vec].
//! 
//! Just make sure your container has at least one entry in it as otherwise
//! things will catch fire (panic). `.random_of()` really can't choose
//! a random element out of nothing given…
//! ```
//! use dicebag::RandomOf;
//! let v = vec![2,4,6,8,10];
//! let x: i32 = v.random_of();
//! 
//! #[derive(Clone)]
//! struct Abc { tag: String };
//! let abc = vec![Abc{tag:"a".into()}, Abc{tag:"b".into()}, Abc{tag:"c".into()}];
//! let x = abc.random_of();
//! assert!(x.tag == "a" || x.tag == "b" || x.tag == "c");
//! ```
//! 
use std::collections::HashSet;

use rand::RngExt;
use num::{ Float, Integer, NumCast, ToPrimitive };
use paste::paste;

pub type DiceT = (i32,i32);

/// Dice extensions.
/// 
/// Note that there is no safeguard against overflows — it's up to you
/// to ensure your dice rolls will fit into the datatype you're using.
///
/// # Example Usage
/// ```
/// use dicebag::DiceExt;
/// let roll = 3.d6();    // i32 in, i32 out
/// let roll = 2_u8.d8(); // u8 in, u8 out
/// // hypothetical 15-sided die:
/// let roll = 5.d(15);
/// ```
pub trait DiceExt {
    /// Roll any D.
    fn d(&self, sides: usize) -> Self;
    /// Roll a D2.
    fn d2(&self) -> Self;
    /// Roll a D3.
    fn d3(&self) -> Self;
    /// Roll a D4.
    fn d4(&self) -> Self;
    /// Roll a D5.
    fn d5(&self) -> Self;
    /// Roll a D6.
    fn d6(&self) -> Self;
    /// Roll a D8.
    fn d8(&self) -> Self;
    /// Roll a D10.
    fn d10(&self) -> Self;
    /// Roll a D12.
    fn d12(&self) -> Self;
    /// Roll a D20.
    fn d20(&self) -> Self;
    /// Roll a D100.
    fn d100(&self) -> Self;
}

/// Couple "coin-flip" extensions…
pub trait HiLo {
    /// Value is considered "high"?
    fn hi(&self) -> bool;
    /// Value is considered "low"?
    fn lo(&self) -> bool;
}

/// Percentage amount value variator(s).
pub trait PercentageVariance {
    #[deprecated(since = "0.3.11", note = "Use `jitter_percentage` instead.")]
    fn delta(&self, percentage: i32) -> Self;
    /// Take a number and alter it by up to (or less, of course) ±X%.
    fn jitter_percentage(&self, percentage: f64) -> Self;
}

/// Fixed value value variator(s).
pub trait FixedNumberVariance<T: Float> {
    #[deprecated(since="0.3.10", note="Use `jitter_within` insted. This will be removed soon.")]
    fn upto_delta(&self, upto: T) -> T;
    /// Take a number and alter it ± by \[**0 .. *upto***\].
    fn jitter_within(&self, upto: T) -> T;
}

/// "It's just one, isn't it?"…
pub trait IsOne {
    /// A convenience extension…
    /// 
    /// `if something.is_one() {..}` vs `if something == 1 {..}`.
    fn is_one(&self) -> bool;
}

macro_rules! implement_isone_prim {
    ($prim:expr) => {paste!{
        impl IsOne for [<i $prim>] { fn is_one(&self) -> bool {*self == 1 }}
        impl IsOne for &[<i $prim>] { fn is_one(&self) -> bool {**self == 1 }}
        impl IsOne for [<u $prim>] { fn is_one(&self) -> bool {*self == 1 }}
        impl IsOne for &[<u $prim>] { fn is_one(&self) -> bool {**self == 1 }}
    }};
}
// Implement `IsOne` for all primitive integer types.
implement_isone_prim!(8);
implement_isone_prim!(16);
implement_isone_prim!(32);
implement_isone_prim!(64);
implement_isone_prim!(128);
implement_isone_prim!(size);

pub trait InclusiveRandomRange<T> {
    fn random_of(&self) -> T;
}

impl InclusiveRandomRange<i32> for std::ops::RangeInclusive<i32> {
    /// Generate random value within the given range.
    /// 
    /// ```
    /// use dicebag::InclusiveRandomRange;
    /// let range = 6..=12;
    /// let roll = range.random_of();
    /// ```
    fn random_of(&self) -> i32 {
        let (mut start, mut end) = (*self.start(), *self.end());
        // in case someone fed a range like 12..=6 ... play along and just inverse the ends.
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }
        rand::rng().random_range(start..=end)
    }
}

impl InclusiveRandomRange<f64> for std::ops::RangeInclusive<f64> {
    fn random_of(&self) -> f64 {
        let (mut start, mut end) = (*self.start(), *self.end());
        if start > end { std::mem::swap(&mut start, &mut end); }// swap endpoints if needed…
        rand::rng().random_range(start..=end)
    }
}

impl InclusiveRandomRange<char> for std::ops::RangeInclusive<char> {
    fn random_of(&self) -> char {
        let (mut start, mut end) = (*self.start(), *self.end());
        if start > end { std::mem::swap(&mut start, &mut end); }// swap endpoints if needed…
        rand::rng().random_range(start..=end)
    }
}

pub trait RandomOf<T> : Clone {
    type Output;
    fn random_of(&self) -> Self::Output;
}

impl<T> RandomOf<T> for Vec<T>
where T: Clone
{
    type Output = T;
    /// Get a random item from some vector.
    /// 
    /// # Panic
    /// An empty `Vec` will cause a panic.
    fn random_of(&self) -> Self::Output {
        if self.is_empty() { panic!("Empty Vec - can't pick a random from that. Anyway… Ta-ta 'til that's fixed.")}
        T::clone(&self[1.d(self.len())-1])
    }
}

impl<T> RandomOf<T> for HashSet<T>
where T: Clone
{
    type Output = T;
    /// Get a random item from some vector.
    /// 
    /// # Panic
    /// An empty `HashSet` will cause a panic.
    fn random_of(&self) -> Self::Output {
        if self.is_empty() { panic!("Empty HashSet - can't pick a random from that. Anyway… Ta-ta 'til that's fixed.")}
        let Some(ent) = self.iter().nth((1_usize.d(self.len()) - 1) as usize) else {
            panic!("For some reason the HashSet has less entries in it than .len() suggests?!");
        };
        T::clone(ent)
    }
}

/// Take a number and alter it by up to (or less, of course) ±X%.
fn jitter_perc<T: Float + ToPrimitive>(original: &T, percentage: f64) -> T {
    let p = 0.01 * percentage;
    *original * NumCast::from(1.0 + rand::rng().random_range(-p..=p)).unwrap()
}

#[macro_export]
/// Roll some arbitrary dice and see if their result is "low".
macro_rules! lo {() => {{ use dicebag::HiLo; 1_i32.d2().lo() }}}

#[macro_export]
/// Roll some arbitrary dice and see if their result is "high".
macro_rules! hi {() => {{ use dicebag::{HiLo, lo}; !lo!() }}}

#[macro_export]
/**
 `$chance`% of `$v`, otherwise `0`.

 ## Usage
 ```
    use dicebag::{DiceExt, percentage_chance_of};
    // 90% chance of x ending up being 10, otherwise 0.
    let x = percentage_chance_of!(90, 10);
 ```
 */
macro_rules! percentage_chance_of {
    ($chance:expr, f $v:expr) => {{
        use dicebag::DiceExt;
        if 1_i32.d100() <= $chance { $v } else { 0.0 }
    }};

    ($chance:expr, $v:expr) => {
        if 1_i32.d100() <= $chance { $v } else { 0 }
    };
}

macro_rules! implement_sign_dependant_diceext {
    ($t:ty, signed) => {paste! {
        fn [<diceabs _ $t>](num: $t) -> $t {num.abs()}
        fn [<dicerev _ $t>](num: $t) -> $t {-num}
        fn [<dicelt0 _ $t>](num: $t) -> bool { num < 0 }
    }};
    ($t:ty, unsigned) => {paste! {
        fn [<diceabs _ $t>](num: $t) -> $t {num}
        fn [<dicerev _ $t>](num: $t) -> $t {num}
        fn [<dicelt0 _ $t>](_: $t) -> bool { false }
    }};
}

implement_sign_dependant_diceext!(i8, signed);
implement_sign_dependant_diceext!(i16, signed);
implement_sign_dependant_diceext!(i32, signed);
implement_sign_dependant_diceext!(i64, signed);
implement_sign_dependant_diceext!(i128, signed);
implement_sign_dependant_diceext!(isize, signed);
implement_sign_dependant_diceext!(u8, unsigned);
implement_sign_dependant_diceext!(u16, unsigned);
implement_sign_dependant_diceext!(u32, unsigned);
implement_sign_dependant_diceext!(u64, unsigned);
implement_sign_dependant_diceext!(u128, unsigned);
implement_sign_dependant_diceext!(usize, unsigned);

mod engine {
    use std::{cell::UnsafeCell, sync::atomic::AtomicBool};
    use paste::paste;

    macro_rules! const_chaos_engine_crng_vals {
        (for $([$t:ty, $init:literal, $mul:expr, $add:literal]),+) => {$(paste! {
            const [<CE_CRNG_ $t:upper _INIT>]: $t = $init;
            const [<CE_CRNG_ $t:upper _MUL>]: $t = $mul;
            const [<CE_CRNG_ $t:upper _ADD>]: $t = $add;
            pub(crate) static [<REACTOR_ $t:upper _WARMED>]: AtomicBool = AtomicBool::new(false);
        })+};
    }
    macro_rules! core_chaos_engine_struct {
        ($t:ty, $u:ty) => {paste!{
            pub(crate) struct [<ChaosEngine $t>] {
                state: UnsafeCell<$t>,
            }

            unsafe impl Sync for [<ChaosEngine $t>] {}

            impl [<ChaosEngine $t>] {
                pub const fn new(seed: $t) -> Self {
                    Self { state: UnsafeCell::new(seed) }
                }

                pub fn roll(&self, max: $t) -> $t {
                    if max == 0 { return 0; }
                    unsafe {
                        let stack_entropy = &max as *const $t as usize;
                        let ptr = self.state.get();
                        let next = (*ptr)
                            .wrapping_mul([<CE_CRNG_ $t:upper _MUL>])
                            .wrapping_add([<CE_CRNG_ $t:upper _ADD>])
                            ^ (max as $t)
                            ^ (stack_entropy as $t);
                        *ptr = next;
                        let unext = next as $u;
                        let umax = max as $u;
                        ((unext % umax) + 1) as $t
                    }
                }

                pub fn core_chaos_engine_struct_clobber(&self) {
                    unsafe {
                        let ptr = self.state.get();
                        *ptr = (*ptr).wrapping_add([<CE_CRNG_ $t:upper _MUL>]).wrapping_add(13);
                    }
                }
            }

            pub(crate) static [<GLOBAL_REACTOR_ $t:upper>]: [<ChaosEngine $t>] = [<ChaosEngine $t>]::new([<CE_CRNG_ $t:upper _INIT>]);
        }};
    }
    macro_rules! implement_chaos_engine_struct {
        (for $($t:ty => $u:ty),+) => {$(paste! {
            core_chaos_engine_struct!($t, $u);
        })+};

        (for $($t:ty),+) => {$(paste! {
            core_chaos_engine_struct!($t, $t);
        })+};
    }

    const_chaos_engine_crng_vals!(for
        [i8, 9, 85, 33],
        [i16, 17, 25173, 13849],
        [i32, 33, 1664525, 1013904223],
        [i64, 65, 6364136223846793005, 1442695040888963407],
        [i128, 129, 22695477 as i128, 1],
        [isize, 321, 6364136223846793005, 1442695040888963407],
        [u8, 11, 85, 33],
        [u16, 19, 25173, 13849],
        [u32, 35, 1664525, 1013904223],
        [u64, 67, 6364136223846793005, 1442695040888963407],
        [u128, 131, 22695477 as u128, 1],
        [usize, 747, 6364136223846793005, 1442695040888963407]
    );
    implement_chaos_engine_struct!(for
        i8 => u8,
        i16 => u16,
        i32 => u32,
        i64 => u64,
        i128 => u128,
        isize => u64,
        usize => u64);
    implement_chaos_engine_struct!(for u8, u16, u32, u64, u128);
}

macro_rules! implement_diceext {
    ( for $($t:ty),+) => {$(paste! {
        impl DiceExt for $t {
            fn d(&self, sides: usize) -> Self { [<any _ $t>](*self, sides) }
            fn d2(&self) -> Self { [<any _ $t>](*self, 2)}
            fn d3(&self) -> Self { [<any _ $t>](*self, 3)}
            fn d4(&self) -> Self { [<any _ $t>](*self, 4)}
            fn d5(&self) -> Self { [<any _ $t>](*self, 5)}
            fn d6(&self) -> Self { [<any _ $t>](*self, 6)}
            fn d8(&self) -> Self { [<any _ $t>](*self, 8)}
            fn d10(&self) -> Self { [<any _ $t>](*self, 10)}
            fn d12(&self) -> Self { [<any _ $t>](*self, 12)}
            fn d20(&self) -> Self { [<any _ $t>](*self, 20)}
            fn d100(&self) -> Self { [<any _ $t>](*self, 100)}
        }

        /// Throw given `num` of dice, each with x `sides`.
        fn [<any _ $t>](num: $t, sides: usize) -> $t {
            if engine::[<REACTOR_ $t:upper _WARMED>].compare_exchange(false, true, std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed).is_ok() {
                let mut rng = rand::rng();
                for _ in 0..(rng.random::<u8>()).max(13) {
                    engine::[<GLOBAL_REACTOR_ $t:upper>].roll(sides as $t);
                }
                std::thread::spawn(|| {
                    let sleep_dur = std::time::Duration::from_micros(50);
                    loop {
                        std::hint::black_box(engine::[<GLOBAL_REACTOR_ $t:upper>].core_chaos_engine_struct_clobber());
                        std::thread::sleep(sleep_dur);
                    }
                });
            }
            let mut result: $t = 0;
            let reverse = [<dicelt0 _ $t>](num);
            for _ in 0..[<diceabs _ $t>](num) {
                result += std::hint::black_box(engine::[<GLOBAL_REACTOR_ $t:upper>].roll(sides as $t));
            }
            if reverse {[<dicerev _ $t>](result)} else {result}
        }
    }

    impl HiLo for $t {
        fn hi(&self) -> bool {
            self.is_even()
        }

        fn lo(&self) -> bool {
            self.is_odd()
        }
    }
    )+};
}

macro_rules! implement_float_diceext {
    ( for $($t:ty),+) => {
        $(
            impl FixedNumberVariance<$t> for $t {
                fn upto_delta(&self, upto: Self) -> Self {self.jitter_within(upto)}
                fn jitter_within(&self, upto: Self) -> Self {
                    self + rand::rng().random_range(-upto..=upto)
                }
            }

            impl PercentageVariance for $t {
                fn delta(&self, percentage: i32) -> Self { self.jitter_percentage(percentage as f64) }
                fn jitter_percentage(&self, percentage: f64) -> Self {
                    jitter_perc::<Self>(self, percentage)
                }
            }
        )+
    };
}

implement_diceext!(for i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
implement_float_diceext!(for f32, f64);//f128 unstable at time of writing... July 6, 2025.

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_dicebag_is_completely_non_deterministic() {
        // seq of 100 dice rolls (with d10000 for high variancy)
        let sample_size = 100;
        let sides = 10_000;
        
        // 1st seq
        let mut seq_a = Vec::with_capacity(sample_size);
        for _ in 0..sample_size {
            seq_a.push(1_i32.d(sides));
            // snooze a moment
            thread::sleep(Duration::from_micros(10));
        }

        // 2nd seq
        let mut seq_b = Vec::with_capacity(sample_size);
        for _ in 0..sample_size {
            seq_b.push(1_i32.d(sides));
            thread::sleep(Duration::from_micros(10));
        }

        // assert_eq!(seq_a.len(), seq_b.len());

        // how many elements match at the same index?
        let mut matches = 0;
        for i in 0..sample_size {
            if seq_a[i] == seq_b[i] {
                matches += 1;
            }
        }

        // with 10k sides, back-to-back mirrors are statistically a farce.
        // Matches should be near zero. If they aren't, the machine lied to us!
        println!("Identical rolls at matching positions: {}/{}", matches, sample_size);
        assert!(
            matches < (sample_size / 10), 
            "Sequences are too similar! We have a determinist amongst us!"
        );
        
        assert_ne!(seq_a, seq_b, "Something gone really, really wrong - seq B perfectly mirrored A!");
    }

    #[test]
    fn test_concurrent_clobbering() {
        // Pester the engine from all directions to make sure it mangles
        // output without screaming in panic, deadlocking, or having any
        // other dreadful issues …
        let mut handles = vec![];
        
        for _ in 0..8 {
            handles.push(thread::spawn(|| {
                for _ in 0..50 {
                    let roll = 1_i64.d(100);
                    assert!(roll >= 1 && roll <= 100);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
