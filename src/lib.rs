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
use rand::Rng;
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

pub trait RandomOf<T> : Clone {
    type Output;
    fn random_of(&self) -> Self::Output;
}

impl<T> RandomOf<T> for Vec<T>
where T: Clone
{
    type Output = T;
    /// Get a random item from some vector.
    fn random_of(&self) -> Self::Output {
        if self.is_empty() { panic!("Nee-neer - pointing finger at dev(s). Empty Vec - can't pick a random from that. Anyway… Ta-ta 'til that's fixed.")}
        T::clone(&self[1.d(self.len())-1]).clone()
    }
}

/// Take a number and alter it by up to (or less, of course) ±X%.
fn jitter_perc<T: Float + ToPrimitive>(original: &T, percentage: f64) -> T {
    let p = 0.01 * percentage;
    *original * NumCast::from(1.0 + rand::rng().random_range(-p..=p)).unwrap()
}

#[macro_export]
/// Roll some arbitrary dice and see if their result is "low".
macro_rules! lo {() => { 1_i32.d2().lo() }}

#[macro_export]
/// Roll some arbitrary dice and see if their result is "high".
macro_rules! hi {() => {!lo!()}}

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
    ($chance:expr, $v:expr) => {
        if 1.d100() <= $chance { $v } else { 0 }
    }
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
        fn [<dicelt0 _ $t>](num: $t) -> bool { false }
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

macro_rules! implement_diceext {
    ( for $($t:ty),+) => {
        $(
            paste! {
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
                    let mut result: $t = 0;
                    let reverse = [<dicelt0 _ $t>](num);
                    let mut rng = rand::rng();
                    for _ in 0..[<diceabs _ $t>](num) {
                        result += rng.random_range(1..=(sides as $t));
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
        )+
    };
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

implement_diceext!(for i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize);
implement_float_diceext!(for f32, f64);//f128 unstable at time of writing... July 6, 2025.

#[cfg(test)]
mod dice_tests {
    use crate::{DiceExt, InclusiveRandomRange, RandomOf, percentage_chance_of};

    /// See that D6 rolls stay within range.
    #[test]
    fn d6_stay_in_range() {
        for _ in 0..10_000 {
            let d = 1.d6();
            assert!(d >= 1 && d <= 6);
        }
    }

    /// See that d(97) rolls stay within range.
    #[test]
    fn d97_stay_in_range() {
        for _ in 0..10_000 {
            let d = 1.d(97);
            assert!(d >= 1 && d <= 97);
        }
    }

    #[test]
    fn chance_macro_works() {
        for _ in 0..20 {
            println!("{}", percentage_chance_of!(5, 50))
        }
    }

    #[test]
    fn random_of_vec() {
        let vs = vec![&1,&2,&3,&4,&5];
        let v = vs.random_of();
        assert_ne!(0, *v);
    }

    #[test]
    fn random_of_f64() {
        let vs = 0.5..=2.0;
        for _ in 0..100_001 {
            let v = vs.random_of();
            assert!(vs.contains(&v))
        }
    }
}
