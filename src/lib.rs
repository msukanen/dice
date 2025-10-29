//! Dice rolling!
use rand::Rng;
use num::{ Float, Integer, NumCast, ToPrimitive };
use paste::paste;

pub type DiceT = (i32,i32);

/// Dice extensions.
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

pub trait HiLo {
    /// Value is considered "high"?
    fn hi(&self) -> bool;
    /// Value is considered "low"?
    fn lo(&self) -> bool;
}

/// Percentage amount value variator(s).
pub trait PercentageVariance {
    /// Take a number and alter it by up to (or less, of course) ±X%.
    fn delta(&self, percentage: i32) -> Self;
}

/// Fixed value value variator(s).
pub trait FixedNumberVariance<T: Float> {
    /// Take a number and alter it ± by \[**0 .. *upto***\], and return result.
    fn upto_delta(&self, upto: T) -> T;
}

pub trait IsOne {
    fn is_one(&self) -> bool;
}

impl IsOne for i32 {
    fn is_one(&self) -> bool {
        *self == 1
    }
}

pub trait InclusiveRandomRange {
    fn random_of(&self) -> i32;
}

impl InclusiveRandomRange for std::ops::RangeInclusive<i32> {
    fn random_of(&self) -> i32 {
        rand::rng().random_range(*self.start()..=*self.end())
    }
}

/// Take a number and alter it by up to (or less, of course) ±X%.
fn delta_p<T: Float + ToPrimitive>(original: &T, percentage: i32) -> T {
    let p = 0.01 * percentage as f64;
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
                fn upto_delta(&self, upto: Self) -> Self {
                    self + rand::rng().random_range(-upto..=upto)
                }
            }

            impl PercentageVariance for $t {
                fn delta(&self, percentage:i32) -> Self { delta_p::<Self>(self, percentage) }
            }
        )+
    };
}

implement_diceext!(for i32, i64, i128, u32, u64, u128, usize);
implement_float_diceext!(for f32, f64);//f128 unstable at time of writing... July 6, 2025.

#[cfg(test)]
mod dice_tests {
    use crate::{DiceExt, percentage_chance_of};

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
}
