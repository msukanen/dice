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
//! Comes with two convenience macros that dynamically infer their return 
//! type based on your assignment context:
//!
//! ```
//! use dicebag::{DiceExt, HiLo, lo, hi};
//! 
//! // The macros adapt seamlessly to left-side type.
//! let is_high_u8: u8 = hi!(); 
//! let is_low_i64: i64 = lo!();
//! 
//! if lo!() { /* do something if result was "low" */ }
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
//! A trait to get some random entry from e.g. [Vec] or [HashSet]
//! (or some other sequential whatsoever,
//!  if you decide to explicit impl the `RandomOf<T>` for it).
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
//! ## `KeyedRandomOf<K,T>`
//! 
//! As per `RandomOf<T>`, but for e.g. [HashMap]s.
//! 
//! Just make sure your container has at least one entry in it as otherwise
//! things will catch fire (panic). `.random_of()` really can't choose
//! a random element out of nothing given…
//! 
use std::{collections::{HashMap, HashSet}, hash::Hash};

use num::{ Float, Integer };
use paste::paste;
use serde::{Deserialize, Serialize, de::{Error, MapAccess, SeqAccess, Visitor}, ser::{SerializeSeq, SerializeStruct}};

pub type DiceT = (i32,i32);

/// Dice roll matrix mod for e.g. serde parsing, etc.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum DiceRollMatrixMod {
    Add(u8),
    /// `Div` ignores decimals entirely.
    Div(u8),
    /// `DivUp` "rounds" upward if dividing ends up with decimals.
    DivUp(u8),
    Mul(u8),
    Sub(u8),
}

impl <'de> Deserialize<'de> for DiceRollMatrixMod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
    {
        struct ModVis;
        impl<'de> Visitor<'de> for ModVis {
            type Value = DiceRollMatrixMod;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a case-insensitive dice modifier thingy")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where A: MapAccess<'de>,
            {
                if let Some((key, value)) = map.next_entry::<String, u8>()? {
                    match key.to_lowercase().as_str() {
                        "add" => Ok(DiceRollMatrixMod::Add(value)),
                        "div" => Ok(DiceRollMatrixMod::Div(value)),
                        "divup"|"div_up" => Ok(DiceRollMatrixMod::DivUp(value)),
                        "mul" => Ok(DiceRollMatrixMod::Mul(value)),
                        "sub" => Ok(DiceRollMatrixMod::Sub(value)),
                        _ => Err(A::Error::unknown_field(&key, &["add","div","divup","div_up","mul","sub"]))
                    }
                } else {
                    Err(A::Error::custom("empty modifier thingy"))
                }
            }
        }

        deserializer.deserialize_map(ModVis)
    }
}

trait DiceRollMatrixModifier {
    fn drmm(&self, drmm: DiceRollMatrixMod) -> i32;
}

impl DiceRollMatrixModifier for i32 {
    fn drmm(&self, drmm: DiceRollMatrixMod) -> i32 {
        match drmm {
            DiceRollMatrixMod::Add(v) => self + v as i32,
            DiceRollMatrixMod::Div(v) => self / v as i32,
            DiceRollMatrixMod::DivUp(v) => {
                let v = v as i32;
                (self + v - 1) / v
            }
            DiceRollMatrixMod::Mul(v) => self * v as i32,
            DiceRollMatrixMod::Sub(v) => self - v as i32,
        }
    }
}

/// Dice roll matrix for e.g. serde parsing, etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiceRollMatrix {
    Exact { value: i32 },
    Percentage(u8), // de: "123%" -> 123_u8, ser: 123u8 -> "123%"
    Chance(u8, Box<DiceRollMatrix>),
    Multi(u8, u8),
    MultiWithMod(u8, u8, DiceRollMatrixMod),
}

impl DiceRollMatrix {
    pub fn roll(&self) -> i32 {
        match self {
            Self::Exact { value } => *value as i32,
            Self::Multi(num, sides) => (*num).d(*sides as usize) as i32,
            Self::MultiWithMod(num, sides, drmm) =>
                Self::Multi(*num, *sides).roll().drmm(*drmm),
            Self::Percentage(p) => if 1.d100() <= *p { 1 } else { 0 },
            Self::Chance(p, drm) =>
                if 1.d100() > *p { 0 }
                else { drm.roll() },
        }
    }
}

impl From<i32> for DiceRollMatrix {
    fn from(value: i32) -> Self {
        Self::Exact { value }
    }
}

impl From<u8> for DiceRollMatrix {
    fn from(value: u8) -> Self {
        Self::Exact { value: value as i32 }
    }
}

impl From<&DiceRollMatrix> for bool {
    fn from(value: &DiceRollMatrix) -> Self {
        value.roll() > 0
    }
}

impl <'de> Deserialize<'de> for DiceRollMatrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        struct DRMVis;
        impl <'de> Visitor<'de> for DRMVis {
            type Value = DiceRollMatrix;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("dice roll matrix thingy")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where E: Error,
            {
                if let Some(percstr) = v.strip_suffix('%') {
                    let n = percstr.parse::<u8>().map_err(E::custom)?;
                    return Ok(DiceRollMatrix::Percentage(n));
                }
                Err(E::custom(format!("'{v}' is an invalid string from DiceRollMatrix")))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where E: Error,
            {
                if v <= u8::MAX as u64 {
                    Ok(DiceRollMatrix::Exact { value: v as i32 })
                } else {
                    Err(E::custom(format!("'{v}' is too large for u8")))
                }
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where A: MapAccess<'de>,
            {
                let (key, raw): (String, serde_json::Value) =
                    map.next_entry()?.ok_or_else(|| A::Error::custom("expceted a single-field object"))?;

                match key.as_str() {
                    "value" => {
                        let v: u8 = serde_json::from_value(raw).map_err(A::Error::custom)?;
                        Ok(DiceRollMatrix::Exact { value: v as i32 })
                    }

                    "chance" => {
                        // expect: [pct, <DiceRollMatrix>]
                        let arr: Vec<serde_json::Value> = serde_json::from_value(raw).map_err(A::Error::custom)?;
                        if arr.len() != 2 {
                            return Err(A::Error::custom("chance must be [pct, matrix]"));
                        }

                        let pct: u8 = serde_json::from_value(arr[0].clone()).map_err(A::Error::custom)?;
                        let inner: DiceRollMatrix = serde_json::from_value(arr[1].clone()).map_err(A::Error::custom)?;
                        Ok(DiceRollMatrix::Chance(pct, Box::new(inner)))
                    }

                    "range" => {
                        let arr: Vec<serde_json::Value> = serde_json::from_value(raw).map_err(A::Error::custom)?;
                        if arr.len() != 2 {
                            return Err(A::Error::custom("range must be [a, b]"));
                        }
                        let a: i32 = serde_json::from_value(arr[0].clone()).map_err(A::Error::custom)?;
                        let b: i32 = serde_json::from_value(arr[1].clone()).map_err(A::Error::custom)?;
                        let (modf, delta) = if a > b {
                            log::warn!("Range values require an U-turn from [{a},{b}] to [{b},{a}]. \
                            See to changing that although we'll let that pass… for now.");
                            let delta = a - b;
                            (a - 1 - delta, delta as u8)
                        } else {
                            let delta = b - a;
                            (b - 1 - delta, delta as u8)
                        };
                        if modf.abs() > u8::MAX as i32 {
                            return Err(A::Error::custom(format!("Modifier {modf} doesn't fit into u8…")));
                        }
                        Ok(if delta == 0 {
                            DiceRollMatrix::Exact { value: 0 }
                        } else {
                            DiceRollMatrix::MultiWithMod(1, delta, if modf < 0 { DiceRollMatrixMod::Sub(modf as u8)} else { DiceRollMatrixMod::Add(modf as u8) })
                        })
                    }

                    _ => Err(A::Error::custom(format!("unknown offender '{key}' in DiceRollMatrix")))
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: SeqAccess<'de>,
            {
                let a: u8 = seq.next_element()?.ok_or_else(|| A::Error::custom("missing 1st element"))?;
                let b: u8 = seq.next_element()?.ok_or_else(|| A::Error::custom("missing 2nd element"))?;
                if let Some(modf) = seq.next_element::<DiceRollMatrixMod>()? {
                    Ok(DiceRollMatrix::MultiWithMod(a,b,modf))
                } else {
                    Ok(DiceRollMatrix::Multi(a,b))
                }
            }
        }

        deserializer.deserialize_any(DRMVis)
    }
}

impl Serialize for DiceRollMatrix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer
    {
        match self {
            Self::Exact { value } => {
                let mut st = serializer.serialize_struct("Exact", 1)?;
                st.serialize_field("value", value)?;
                st.end()
            }

            Self::Multi(a,b) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element(a)?;
                seq.serialize_element(b)?;
                seq.end()
            }

            Self::MultiWithMod(a,b,m) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element(a)?;
                seq.serialize_element(b)?;
                seq.serialize_element(m)?;
                seq.end()
            }

            Self::Percentage(p) => serializer.serialize_str(&format!("{p}%")),

            Self::Chance(p, drm) => {
                let mut st = serializer.serialize_struct("Chance", 1)?;
                st.serialize_field("chance", &(p, drm))?;
                st.end()
            }
        }
    }
}

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
        impl IsOne for [<i $prim>] { #[inline(always)] fn is_one(&self) -> bool {*self == 1 }}
        impl IsOne for &[<i $prim>] { #[inline(always)] fn is_one(&self) -> bool {**self == 1 }}
        impl IsOne for [<u $prim>] { #[inline(always)] fn is_one(&self) -> bool {*self == 1 }}
        impl IsOne for &[<u $prim>] { #[inline(always)] fn is_one(&self) -> bool {**self == 1 }}
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
        let start = start as i64;
        let end = end as i64;
        let sides = end - start + 1;
        // engine.roll(X) gives 1..=X value. Shift it down.
        (start + (engine::GLOBAL_REACTOR_U64.roll(sides as u64) - 1) as i64) as i32
        // rand::rng().random_range(start..=end)
    }
}

impl InclusiveRandomRange<f64> for std::ops::RangeInclusive<f64> {
    fn random_of(&self) -> f64 {
        let (mut start, mut end) = (*self.start(), *self.end());
        if start > end { std::mem::swap(&mut start, &mut end); }// swap endpoints if needed…

        let raw_bits = engine::GLOBAL_REACTOR_U64.roll(u64::MAX);
        let max_m = (1u64 << 53) - 1; // use 53 bits of mantissa of the f64
        start + ((raw_bits & max_m) as f64 / max_m as f64) * (end - start)
        // rand::rng().random_range(start..=end)
    }
}

impl InclusiveRandomRange<f32> for std::ops::RangeInclusive<f32> {
    fn random_of(&self) -> f32 {
        let (mut start, mut end) = (*self.start(), *self.end());
        if start > end { std::mem::swap(&mut start, &mut end); }// swap endpoints if needed…

        let raw_bits = engine::GLOBAL_REACTOR_U64.roll(u64::MAX);
        let max_m = (1u32 << 24) - 1; // use 24 bits of mantissa of the f32
        start + ((raw_bits as u32 & max_m) as f32 / max_m as f32) * (end - start)
        // rand::rng().random_range(start..=end)
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
            let offt = engine::GLOBAL_REACTOR_U64.roll(range as u64) as u32;
            let maybe = u_start + offt;
            // skip the illegal surrogate gap
            if !(0xD800..=0xDFFF).contains(&maybe) {
                return unsafe { char::from_u32_unchecked(maybe) }
            }
        }
    }
}

pub trait RandomOf<T> : Clone {
    type Output;
    fn random_of(&self) -> Self::Output;
}

pub trait PlainRandomOf : Clone {
    type Output;
    fn random_of() -> Self::Output;
}

pub trait KeyedRandomOf<K,T> : Clone {
    type Output;
    fn random_of(&self) -> Self::Output;
}

pub trait KeyedAllocatorRandomOf<K,T,A> : Clone {
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

impl <T> RandomOf<T> for HashMap<String, T>
where T: Clone
{
    type Output = T;
    fn random_of(&self) -> Self::Output {
        if self.is_empty() { panic!("Empty HashMap - can't pick a random from that. Anyway… Ta-ta 'til that's fixed.")}
        let Some((_,ent)) = self.iter().nth((1_usize.d(self.len()) - 1) as usize) else {
            panic!("For some reason the HashMap has less entries in it than .len() suggests?!");
        };
        T::clone(ent)
    }
}

impl <K,T> KeyedRandomOf<K,T> for HashMap<K,T>
where T: Clone, K: Hash + Clone
{
    type Output = T;
    fn random_of(&self) -> Self::Output {
        if self.is_empty() { panic!("Empty HashMap - can't pick a random from that. Anyway… Ta-ta 'til that's fixed.")}
        let Some((_,ent)) = self.iter().nth((1_usize.d(self.len()) - 1) as usize) else {
            panic!("For some reason the HashMap has less entries in it than .len() suggests?!");
        };
        T::clone(ent)
    }
}

impl <K,T,A> KeyedAllocatorRandomOf<K,T,A> for HashMap<K,T,A>
where
    T: Clone,
    K: Hash + Clone,
    A: Clone,
{
    type Output = T;
    fn random_of(&self) -> Self::Output {
        if self.is_empty() { panic!("Empty HashMap - can't pick a random from that. Anyway… Ta-ta 'til that's fixed.")}
        let Some((_,ent)) = self.iter().nth((1_usize.d(self.len()) - 1) as usize) else {
            panic!("For some reason the HashMap has less entries in it than .len() suggests?!");
        };
        T::clone(ent)
    }
}

#[macro_export]
/// Roll some arbitrary dice and see if their result is "low".
macro_rules! lo {() => {{ use dicebag::{DiceExt, HiLo}; 1_i32.d2().lo() }}}

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

    ($chance:expr, $v:expr) => {{
        use dicebag::DiceExt;
        if 1_i32.d100() <= $chance { $v } else { 0 }
    }};
}

macro_rules! implement_sign_dependant_diceext {
    ($t:ty, signed) => {paste! {
        #[inline(always)] fn [<diceabs _ $t>](num: $t) -> $t {num.abs()}
        #[inline(always)] fn [<dicerev _ $t>](num: $t) -> $t {-num}
        #[inline(always)] fn [<dicelt0 _ $t>](num: $t) -> bool { num < 0 }
    }};
    ($t:ty, unsigned) => {paste! {
        #[inline(always)] fn [<diceabs _ $t>](num: $t) -> $t {num}
        #[inline(always)] fn [<dicerev _ $t>](num: $t) -> $t {num}
        #[inline(always)] fn [<dicelt0 _ $t>](_: $t) -> bool { false }
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
        (for $([$t:ty, $bits:literal bits, $init:literal, $mul:expr, $add:literal]),+ $(,)?) => {$(paste! {
            const [<CE_CRNG_U $bits _INIT>]: $t = $init;
            const [<CE_CRNG_U $bits _MUL>]: $t = $mul;
            const [<CE_CRNG_U $bits _ADD>]: $t = $add;
            pub(crate) static [<REACTOR_U $bits _WARMED>]: AtomicBool = AtomicBool::new(false);
        })+};
    }
    macro_rules! core_chaos_engine_struct {
        ($t:ty, $u:ty, $bits:literal) => {paste!{
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
                        let stack_entropy = &max as *const $t as usize as [<u $bits>];
                        let ptr = self.state.get();
                        let next = (*ptr as [<u $bits>])
                            .wrapping_mul([<CE_CRNG_U $bits _MUL>])
                            .wrapping_add([<CE_CRNG_U $bits _ADD>])
                            ^ (max as [<u $bits>])
                            ^ stack_entropy
                            ^ {
                                let inst = std::time::Instant::now();
                                &inst as *const _ as usize as [<u $bits>]
                            }
                            ;
                        *ptr = next as $t;
                        ((next % (max as [<u $bits>])) + 1) as $t
                    }
                }

                pub fn core_chaos_engine_struct_clobber_(&self) {
                    unsafe {
                        let ptr = self.state.get();
                        *ptr = (*ptr as [<u $bits>]).wrapping_add([<CE_CRNG_U $bits _MUL>]).wrapping_add(13) as $t;
                    }
                }
            }

            pub(crate) static [<GLOBAL_REACTOR_U $bits>]: [<ChaosEngine $t>] = [<ChaosEngine $t>]::new([<CE_CRNG_U $bits _INIT>]);
        }};
    }
    macro_rules! implement_chaos_engine_struct {
        (for $(($t:ty, $bits:literal bits)),+ $(,)?) => {$(paste! {
            core_chaos_engine_struct!($t, $t, $bits);
        })+};
    }

    const_chaos_engine_crng_vals!(for
        [u64, 64 bits, 67, 6364136223846793005, 1442695040888963407],
        [u128, 128 bits, 131, 22695477 as u128, 1],
    );
    implement_chaos_engine_struct!(for
        (u64, 64 bits),
        (u128, 128 bits),
    );
}

macro_rules! implement_diceext {
    ( for $(($t:ty, $bits:literal bits)),+ $(,)?) => {$(paste! {
        impl DiceExt for $t {
            #[inline(always)] fn d(&self, sides: usize) -> Self { [<any _ $t>](*self, sides) }
            #[inline(always)] fn d2(&self) -> Self { [<any _ $t>](*self, 2)}
            #[inline(always)] fn d3(&self) -> Self { [<any _ $t>](*self, 3)}
            #[inline(always)] fn d4(&self) -> Self { [<any _ $t>](*self, 4)}
            #[inline(always)] fn d5(&self) -> Self { [<any _ $t>](*self, 5)}
            #[inline(always)] fn d6(&self) -> Self { [<any _ $t>](*self, 6)}
            #[inline(always)] fn d8(&self) -> Self { [<any _ $t>](*self, 8)}
            #[inline(always)] fn d10(&self) -> Self { [<any _ $t>](*self, 10)}
            #[inline(always)] fn d12(&self) -> Self { [<any _ $t>](*self, 12)}
            #[inline(always)] fn d20(&self) -> Self { [<any _ $t>](*self, 20)}
            #[inline(always)] fn d100(&self) -> Self { [<any _ $t>](*self, 100)}
        }

        /// Throw given `num` of dice, each with x `sides`.
        fn [<any _ $t>](num: $t, sides: usize) -> $t {
            if engine::[<REACTOR_U $bits _WARMED>].compare_exchange(false, true, std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed).is_ok() {
                // chaos seed
                let x = std::time::Instant::now();
                let ptr = &x as *const _ as u64;
                let b = std::time::Instant::now().elapsed().as_nanos() as u64;
                let z = ptr ^ b.rotate_left(7);
                for _ in 0..(z.rotate_left((ptr & 0xF) as u32)).wrapping_rem(512).max(128) {
                    engine::[<GLOBAL_REACTOR_U $bits>].roll(sides as [<u $bits>]);
                }
                std::thread::spawn(|| {
                    let sleep_dur = std::time::Duration::from_micros(25);
                    loop {
                        std::hint::black_box(engine::[<GLOBAL_REACTOR_U $bits>].core_chaos_engine_struct_clobber_());
                        std::thread::sleep(sleep_dur);
                    }
                });
            }
            let mut result: $t = 0;
            for _ in 0..[<diceabs _ $t>](num) {
                result += std::hint::black_box(engine::[<GLOBAL_REACTOR_U $bits>].roll(sides as [<u $bits>]) as $t);
            }
            if [<dicelt0 _ $t>](num) {[<dicerev _ $t>](result)} else {result}
        }
    }

    impl HiLo for $t {
        #[inline(always)] fn hi(&self) -> bool { self.is_even() }
        #[inline(always)] fn lo(&self) -> bool { self.is_odd() }
    }
    )+};
}

macro_rules! implement_float_diceext {
    ( for $($t:ty),+ ) => { $( implement_float_diceext!($t); )+ };
    (f128) => { implement_float_diceext!(reactor U128, u128, 113, f128); };
    ($t:ty) => { implement_float_diceext!(reactor U64, u64, 63, $t); };
    (reactor $r:ident, $type:ty, $mantissa_bits:literal, $t:ty) => {paste!{
        impl FixedNumberVariance<$t> for $t {
            fn upto_delta(&self, upto: Self) -> Self {self.jitter_within(upto)}
            fn jitter_within(&self, upto: Self) -> Self {
                if upto <= 0.0 { return *self; }
                let raw_bits = engine::[<GLOBAL_REACTOR_ $r>].roll($type::MAX);
                let mantissa_bits = ([<$t>]::MANTISSA_DIGITS as u32).min($mantissa_bits);
                let max_mask = ((1 as $type) << mantissa_bits) - 1;
                let scale = (raw_bits & max_mask) as $t / max_mask as $t;
                self + ((scale * 2.0 * upto ) - upto)
            }
        }

        impl PercentageVariance for $t {
            fn delta(&self, percentage: i32) -> Self { self.jitter_percentage(percentage as f64) }
            fn jitter_percentage(&self, percentage: f64) -> Self {
                let p = 0.01 * percentage as $t;
                self.jitter_within( self.abs() * p )
            }
        }
    }};
}

implement_diceext!(for
    (i8, 64 bits),
    (i16, 64 bits),
    (i32, 64 bits),
    (i64, 64 bits),
    (i128, 128 bits),
    (isize, 64 bits),
    (u8, 64 bits),
    (u16, 64 bits),
    (u32, 64 bits),
    (u64, 64 bits),
    (u128, 128 bits),
    (usize, 64 bits),
);
#[cfg(not(feature = "f128-stable"))]
implement_float_diceext!(for f32, f64);//f128 unstable at time of writing... July 6, 2025.
#[cfg(feature = "f128-stable")]
implement_float_diceext!(for f32, f64, f128);
