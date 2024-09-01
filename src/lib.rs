//!
//! Dice rolling!
//! 
use rand::Rng;
use num::{ Float, Integer, NumCast, ToPrimitive };

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
    /// If chance on `d100` matches `of` then return *self*, otherwise return `None`.
    fn chance(&self, of:i32) -> Option<i32>;
}

pub trait HiLo {
    /// Value is considered "high"?
    fn hi(&self) -> bool;
    /// Value is considered "low"?
    fn lo(&self) -> bool;
}

/// Throw given `num` of dice, each with x `sides`.
fn any_i32(num: i32, sides: usize) -> i32 {
    let mut result: i32 = 0;
    let reverse = num < 0;
    for _ in 0..num.abs() {
        result += rand::thread_rng().gen_range(1..=(sides as i32));
    }
    if reverse {-result} else {result}
}

impl DiceExt for i32 {
    fn d(&self, sides: usize) -> Self { any_i32(*self, sides) }
    fn d2(&self) -> Self { any_i32(*self, 2)}
    fn d3(&self) -> Self { any_i32(*self, 3)}
    fn d4(&self) -> Self { any_i32(*self, 4)}
    fn d5(&self) -> Self { any_i32(*self, 5)}
    fn d6(&self) -> Self { any_i32(*self, 6)}
    fn d8(&self) -> Self { any_i32(*self, 8)}
    fn d10(&self) -> Self { any_i32(*self, 10)}
    fn d12(&self) -> Self { any_i32(*self, 12)}
    fn d20(&self) -> Self { any_i32(*self, 20)}
    fn d100(&self) -> Self { any_i32(*self, 100)}
    fn chance(&self, of:i32) -> Option<i32> {
        if 3.d6() as Self <= *self {Some(of)} else {None}
    }
}

impl HiLo for i32 {
    fn hi(&self) -> bool {
        self.is_even()
    }

    fn lo(&self) -> bool {
        self.is_odd()
    }
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

impl FixedNumberVariance<f64> for f64 {
    fn upto_delta(&self, upto: Self) -> Self {
        self + rand::thread_rng().gen_range(-upto..=upto)
    }
}

impl FixedNumberVariance<f32> for f32 {
    fn upto_delta(&self, upto: Self) -> Self {
        self + rand::thread_rng().gen_range(-upto..=upto)
    }
}

/// Take a number and alter it by up to (or less, of course) ±X%.
fn delta_p<T: Float + ToPrimitive>(original: &T, percentage: i32) -> T {
    let p = 0.01 * percentage as f64;
    *original * NumCast::from(1.0 + rand::thread_rng().gen_range(-p..=p)).unwrap()
}

impl PercentageVariance for f32 {
    fn delta(&self, percentage:i32) -> Self { delta_p::<Self>(self, percentage) }
}

impl PercentageVariance for f64 {
    fn delta(&self, percentage:i32) -> Self { delta_p::<Self>(self, percentage) }
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
   // 90% chance of x ending up being 10, otherwise 0.
   let x = chance_of!(90, 10);
 ```
 */
macro_rules! chance_of {
    ($chance:expr, $v:expr) => {
        if 1.d100() <= $chance { $v } else { 0 }
    }
}

#[cfg(test)]
mod dice_tests {
    use crate::DiceExt;

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
            println!("{}", chance_of!(5, 50))
        }
    }
}
