use rand_xoshiro::{Xoshiro512Plus, rand_core::{Rng, SeedableRng}};
use std::{cell::UnsafeCell, sync::OnceLock};

mod diceext; pub use diceext::*;
mod dicermatrix; pub use dicermatrix::*;
mod incl_rr; pub use incl_rr::*;
mod is_one; pub use is_one::*;
mod randomof; pub use randomof::*;
mod variance; pub use variance::*;

const SHARD_COUNT: usize = 4096; // tune for hardware

pub(crate) struct ChaoticDice {
    shards: Box<[UnsafeCell<Xoshiro512Plus>]>,
}

unsafe impl Send for ChaoticDice {}
unsafe impl Sync for ChaoticDice {}

impl ChaoticDice {
    pub(crate) fn new(seed: u64) -> Self {
        let mut shards = Vec::with_capacity(SHARD_COUNT);
        for i in 0..SHARD_COUNT {
            shards.push(UnsafeCell::new(Xoshiro512Plus::seed_from_u64(
                seed.wrapping_add(i as u64),
            )));
        }
        Self {
            shards: shards.into_boxed_slice(),
        }
    }

    #[inline(always)]
    pub(crate) fn roll_one(&self, sides: u64) -> u64 {
        debug_assert!(sides >= 1, "Dice must have at least 1 side");
        let idx = fast_shard_index();
        // (unsafe { &mut *self.shards[idx].get() }.next_u64() % sides) + 1
        let rng = unsafe { &mut *self.shards[idx].get() };
        let x = rng.next_u64();
        let mut m = (x as u128) * (sides as u128);
        let mut l = m as u64;
        if l < sides {
            let t = sides.wrapping_neg() % sides;
            while l < t {
                m = (rng.next_u64() as u128) * (sides as u128);
                l = m as u64;
            }
        }
        ((m >> 64) as u64) + 1
    }

    #[inline(always)]
    pub(crate) fn roll64(&self, count: u64, sides: u64) -> u64 {
        (0..count).map(|_| self.roll_one(sides)).sum()
    }
}

#[inline(always)]
fn fast_shard_index() -> usize {
    let stack_var = 0u8;
    let addr = &stack_var as *const u8 as usize;

    #[cfg(target_arch = "x86_64")]
    let counter = unsafe { core::arch::x86_64::_rdtsc() } as usize;

    #[cfg(target_arch = "aarch64")]
    let counter = unsafe {
        let val: usize;
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) val, options(nomem, nostack));
        val
    };

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    let counter = addr; // Fallback for exotic architectures    let addr = &stack_var as *const u8 as usize;

    (addr ^ counter) & (SHARD_COUNT - 1)
}

// Global chaotic RNG instance
static CHAOS_DICE: OnceLock<ChaoticDice> = OnceLock::new();

pub(crate) fn chaos_dice() -> &'static ChaoticDice {
    fn seed_from_time() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
    CHAOS_DICE.get_or_init(|| ChaoticDice::new(seed_from_time()))
}

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

// #[cfg(all(test, feature = "test-xoshiro-only"))]
// mod dice_xoshiro_tests {
//     use super::*;

//     macro_rules! rolly_polly {
//         ([$($t:tt),*]) => {paste::paste!{$(
//             rolly_polly!(type [<u $t>]);
//             rolly_polly!(type [<i $t>]);
//         )+}};
//         (type $t:tt) => {paste::paste!{
//             _ = [<5_ $t>].d(6);
//             _ = [<3_ $t>].d(20);
//             _ = [<1_ $t>].d(1337);
//         }};
//     }

//     #[test]
//     fn threaded_8() {
//         use std::thread;
//         _ = env_logger::try_init();

//         let mut handles = vec![];
//         for _ in 0..1_024 {
//             handles.push(thread::spawn(move || {
//                 for _ in 0..10_000 {
//                     rolly_polly!([8,16,32,64,128,size]);
//                 }
//             }));
//         }

//         for h in handles {
//             h.join().unwrap();
//         }
//     }
// }
