use rand::Rng;
use num::{ Float, Integer, NumCast };

/**
 Dice extensions.
 */
pub trait DiceExt {
    /// A d2.
    fn d2(&self) -> Self;
    /// A d3.
    fn d3(&self) -> Self;
    /// A d4.
    fn d4(&self) -> Self;
    /// A d5.
    fn d5(&self) -> Self;
    /// A d6.
    fn d6(&self) -> Self;
    /// A d8.
    fn d8(&self) -> Self;
    /// A d10.
    fn d10(&self) -> Self;
    /// A d12.
    fn d12(&self) -> Self;
    /// A d20.
    fn d20(&self) -> Self;
    /// A d100.
    fn d100(&self) -> Self;
    /// If chance on `d100` matches `of` then return self, otherwise return None.
    fn chance(&self, of:i32) -> Option<i32>;
}

pub trait HiLo {
    fn hi(&self) -> bool;
    fn lo(&self) -> bool;
}

/**
 Throw given `num` of dice, each with x `sides`.
 */
fn any_i32(num: i32, sides:u8) -> i32 {
    let mut result: i32 = 0;
    let reverse = num < 0;
    for _ in 0..num.abs() {
        result += rand::thread_rng().gen_range(1..=(sides as i32));
    }
    if reverse {-result} else {result}
}

impl DiceExt for i32 {
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

/**
 Value variators.
 */
pub trait VarianceExt {
    /**
     Take a number and alter it by up to (or less, of course) +/- X%.
    */
    fn delta(&self, percentage:i32) -> Self;
}

/**
 Take a number and alter it by up to (or less, of course) +/- X%.
 */
fn delta<T: Float>(original:&T, percentage:i32) -> T {
    let r = rand::thread_rng().gen_range(0..=(percentage*2)) - percentage;
    let d = 1.0 + 0.01 * r as f64;
    *original * NumCast::from(d).unwrap()
}

impl VarianceExt for f32 {
    fn delta(&self, percentage:i32) -> Self { delta::<Self>(self, percentage) }
}

impl VarianceExt for f64 {
    fn delta(&self, percentage:i32) -> Self { delta::<Self>(self, percentage) }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
