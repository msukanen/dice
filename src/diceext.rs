pub type DiceExtSides = usize;

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
    fn d(&self, sides: DiceExtSides) -> Self;
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

macro_rules! impl_dice_notation {
    ([$($bits:tt),*]) => {paste::paste! {
        $(  impl_dice_notation!(type [<u $bits>]);
            impl_dice_notation!(type [<i $bits>]);
        )+
    }};

    (type $t:tt) => {
        impl DiceExt for $t {
            #[inline(always)]
            fn d(&self, sides: DiceExtSides) -> Self {
                crate::chaos_dice().roll64(*self as u64, sides as u64) as $t
            }

            #[inline(always)] fn d2(&self) -> Self { self.d(2) }
            #[inline(always)] fn d3(&self) -> Self { self.d(3) }
            #[inline(always)] fn d4(&self) -> Self { self.d(4) }
            #[inline(always)] fn d5(&self) -> Self { self.d(5) }
            #[inline(always)] fn d6(&self) -> Self { self.d(6) }
            #[inline(always)] fn d8(&self) -> Self { self.d(8) }
            #[inline(always)] fn d10(&self) -> Self { self.d(10) }
            #[inline(always)] fn d12(&self) -> Self { self.d(12) }
            #[inline(always)] fn d20(&self) -> Self { self.d(20) }
            #[inline(always)] fn d100(&self) -> Self { self.d(100) }
        }
    };
}
impl_dice_notation!([8,16,32,64,128,size]);

/// Implement some dice extensions for float types.
macro_rules! implement_float_diceext {
    ([$($t:ty),*]) => { $( implement_float_diceext!($t); )+ };
    (f128) => { implement_float_diceext!(reactor U128, u128, 113, f128); };
    ($t:ty) => { implement_float_diceext!(reactor U64, u64, 63, $t); };
    (reactor $r:ident, $type:ty, $mantissa_bits:literal, $t:ty) => {paste::paste!{
        // TODO: adjust this to simulate "chipped" dice properly.
        fn [<dext_chop_suey_ $t>](what: $t, sides: usize) -> $t {
            if what == 0.0 { return 0.0 }

            let wr = |w| w * 1_usize.d(sides) as $t;
            match what as usize {
                0 => {
                    let primary = 1_usize.d(sides);
                    let antipode = (sides + 1) - primary;
                    let frac_p = what * 100.0;
                    if 1.d100() as $t <= frac_p {
                        (primary as $t + antipode as $t) / 2.0
                    } else {
                        primary as $t
                    }
                },
                x => x.d(sides) as $t + wr(what - x as $t)
            }
        }

        impl DiceExt for $t {
            #[inline(always)] fn d(&self, sides: usize) -> Self { [<dext_chop_suey_ $t>](*self, sides) }
            #[inline(always)] fn d2(&self) -> Self { [<dext_chop_suey_ $t>](*self, 2)}
            #[inline(always)] fn d3(&self) -> Self { [<dext_chop_suey_ $t>](*self, 3)}
            #[inline(always)] fn d4(&self) -> Self { [<dext_chop_suey_ $t>](*self, 4)}
            #[inline(always)] fn d5(&self) -> Self { [<dext_chop_suey_ $t>](*self, 5)}
            #[inline(always)] fn d6(&self) -> Self { [<dext_chop_suey_ $t>](*self, 6)}
            #[inline(always)] fn d8(&self) -> Self { [<dext_chop_suey_ $t>](*self, 8)}
            #[inline(always)] fn d10(&self) -> Self { [<dext_chop_suey_ $t>](*self, 10)}
            #[inline(always)] fn d12(&self) -> Self { [<dext_chop_suey_ $t>](*self, 12)}
            #[inline(always)] fn d20(&self) -> Self { [<dext_chop_suey_ $t>](*self, 20)}
            #[inline(always)] fn d100(&self) -> Self { [<dext_chop_suey_ $t>](*self, 100)}
        }
    }};
}

#[cfg(not(feature = "f128-stable"))]
implement_float_diceext!([f32, f64]);
#[cfg(feature = "f128-stable")]
implement_float_diceext!(f128);//f128 still unstable: 13th Jul 2026
