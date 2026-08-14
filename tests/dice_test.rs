use std::{collections::{HashMap, HashSet}, thread};

use dicebag::*;
use nohash::BuildNoHashHasher;
use serde::Deserialize;

const SPAM_THRESHOLD_SMALL: usize = 10_000;
const SPAM_THRESHOLD: usize = 100_001;// 100,001, because it's such a pretty number

fn comma_sep(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut res = String::new();
    for (i,&b) in bytes.iter().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            res.push(',');
        }
        res.push(b as char);
    }
    res.chars().rev().collect()
}

/// See that D6 rolls stay within range.
#[test]
fn d6_stay_in_range() {
    for _ in 0..SPAM_THRESHOLD_SMALL {
        let d = 1.d6();
        assert!(d >= 1 && d <= 6, "d = {}", d);
    }
}

/// See that d(97) rolls stay within range.
#[test]
fn d97_stay_in_range() {
    let mut ds = HashSet::new();
    let mut i = 0;
    let mut sat_at = -1;
    loop {
        let d = 1.d(97);
        ds.insert(d);
        assert!(d >= 1 && d <= 97, "d = {}", d);
        if ds.len() >= 97 && sat_at < 0 {
            sat_at = i;
        }
        i += 1;
        if i >= 10_000 && ds.len() >= 97 {
            break;
        }
    }
    _ = env_logger::try_init();
    log::debug!("Saturated at #{sat_at} out of (at least) 10,000 loops.");
}

/// See that d(97) rolls stay within range.
#[test]
fn full_saturation() {
    let dees = [2,3,4,5,6,8,10,12,20,100];
    _ = env_logger::try_init();
    for sides in dees {
        let mut ds = HashSet::new();
        let mut i = 1;
        let mut sat_at = -1;
        loop {
            let d = 1.d(sides);
            if sides == 8 {
                log::info!("D8: {d}");
            }
            ds.insert(d);
            assert!(d >= 1 && d <= sides, "d = {}", d);
            if ds.len() >= sides && sat_at < 0 {
                sat_at = i;
            }
            i += 1;
            if i as usize > SPAM_THRESHOLD || ds.len() >= sides {
                break;
            }
        }
        log::debug!("Saturated D{sides} @ #{sat_at} out of (at most) {} loops.", comma_sep(SPAM_THRESHOLD));
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
    for _ in 0..SPAM_THRESHOLD_SMALL {
        let v = vs.random_of();
        assert!(*v >= 1 && *v <= 5, "Hol' a moment! We got an out of bounds, wild {v} amongst us!");
    }
}

#[test]
fn random_of_f64() {
    let vs = 0.5..=2.0;
    for _ in 0..SPAM_THRESHOLD {
        let v = vs.random_of();
        assert!(vs.contains(&v))
    }
}

#[test]
fn dice_roll_modifiers() {
    let json = r#"{
        "something": [1, 10, { "add": 2 }]
    }"#;

    #[derive(Deserialize)]
    struct Something {
        something: DiceRollMatrix,
    }

    let s = match serde_json::from_str::<Something>(json) {
        Ok(s) => s,
        Err(e) => panic!("{e:?}")
    };

    let mut old_roll = 0;
    let mut repeats = 0;
    _ = env_logger::try_init();
    for _ in 0..SPAM_THRESHOLD_SMALL {
        let r = s.something.roll();
        // log::debug!("r = {r}");
        if old_roll == r {
            repeats += 1;
        }
        old_roll = r;
        assert!(r >= 3 && r <= 12, "Roll of {r} is out of bounds of [3..=12]!");
    }

    log::debug!("Repeats: {repeats} out of 10,000")
}

#[test]
fn chance_50perc() {
    let c = DiceRollMatrix::Chance(50, Box::new(DiceRollMatrix::Exact { value: 1 }));
    let mut ones = 0;
    const SPAM: usize = SPAM_THRESHOLD_SMALL * 1_000;
    const FRAC: usize = SPAM / 1_000;
    const LOW: usize = SPAM/2 - FRAC;
    const HIGH: usize = LOW + FRAC * 2;
    for _ in 0..SPAM {
        if c.roll() == 1 {
            ones += 1;
        }
    }
    assert!(ones >= LOW && ones <= HIGH, "Strange number of ones: {}; expected permille ±var from {}", comma_sep(ones), comma_sep(SPAM/2));
    _ = env_logger::try_init();
    log::debug!("Exactly {} '1's out of {} pool of 50% chances.", comma_sep(ones), comma_sep(SPAM));
}

#[test]
fn drm_roundtrip_exact_flat() {
    let json = "5";
    let drm: DiceRollMatrix = serde_json::from_str(json).unwrap();
    assert!(matches!(drm, DiceRollMatrix::Exact { value: 5 }));

    let ser = serde_json::to_string(&drm).unwrap();
    let drm2: DiceRollMatrix = serde_json::from_str(&ser).unwrap();
    assert_eq!(drm2, drm);
}

#[test]
fn drm_roundtrip_exact_struct() {
    let json = r#"{ "value": 7 }"#;
    let drm: DiceRollMatrix = serde_json::from_str(json).unwrap();
    assert!(matches!(drm, DiceRollMatrix::Exact { value: 7 }));

    let ser = serde_json::to_string(&drm).unwrap();
    let drm2: DiceRollMatrix = serde_json::from_str(&ser).unwrap();
    assert_eq!(drm2, drm);
}

#[test]
fn drm_roundtrip_percentage() {
    let json = r#""25%""#;
    let drm: DiceRollMatrix = serde_json::from_str(json).unwrap();
    assert!(matches!(drm, DiceRollMatrix::Percentage(25)));

    let ser = serde_json::to_string(&drm).unwrap();
    let drm2: DiceRollMatrix = serde_json::from_str(&ser).unwrap();
    assert_eq!(drm2, drm);
}

#[test]
fn drm_roundtrip_multi() {
    let json = "[3, 6]";
    let drm: DiceRollMatrix = serde_json::from_str(json).unwrap();
    assert!(matches!(drm, DiceRollMatrix::Multi(3, 6)));

    let ser = serde_json::to_string(&drm).unwrap();
    let drm2: DiceRollMatrix = serde_json::from_str(&ser).unwrap();
    assert_eq!(drm2, drm);
}

#[test]
fn drm_roundtrip_multi_with_mod() {
    let json = r#"[3, 6, { "add": 2 }]"#;
    let drm: DiceRollMatrix = serde_json::from_str(json).unwrap();
    assert!(matches!(drm, DiceRollMatrix::MultiWithMod(3, 6, DiceRollMatrixMod::Add(2))));

    let ser = serde_json::to_string(&drm).unwrap();
    let drm2: DiceRollMatrix = serde_json::from_str(&ser).unwrap();
    assert_eq!(drm2, drm);
}

#[test]
fn drm_roundtrip_chance() {
    let json = r#"{ "chance": [50, { "value": 3 }] }"#;
    let drm: DiceRollMatrix = serde_json::from_str(json).unwrap();
    assert!(
        matches!(&drm, DiceRollMatrix::Chance(50, inner)
            if matches!(**inner, DiceRollMatrix::Exact { value: 3 }))
    );

    let ser = serde_json::to_string(&drm).unwrap();
    let drm2: DiceRollMatrix = serde_json::from_str(&ser).unwrap();
    assert_eq!(drm2, drm);
}

#[test]
fn drm_roundtrip_nested_chance() {
    let json = r#"{ "chance": [40, { "chance": [20, { "value": 9 }] }] }"#;
    let drm: DiceRollMatrix = serde_json::from_str(json).unwrap();

    let ser = serde_json::to_string(&drm).unwrap();
    let drm2: DiceRollMatrix = serde_json::from_str(&ser).unwrap();
    assert_eq!(drm2, drm);
}

/// Pester the engine from all directions to make sure it mangles
/// output without screaming in panic, deadlocking, or having any
/// other dreadful issues …
#[test]
fn test_concurrent_clobbering() {
    let mut handles = vec![];
    
    for _ in 0..100 {
        handles.push(thread::spawn(|| {
            for _ in 0..10_000 {
                let roll = 1_i64.d(100);
                assert!(roll >= 1 && roll <= 100);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn dicebag_is_completely_non_deterministic() {
    use std::{time::Duration, thread};
    
    _ = env_logger::try_init();
    // seq of 100,000 dice rolls
    let sample_size = 100_000;
    let sides = 100;
    
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

    // how many elements match at the same index?
    let mut matches = 0;
    for i in 0..sample_size {
        if seq_a[i] == seq_b[i] {
            matches += 1;
        }
    }

    // ... back-to-back mirrors are statistically a farce.
    // Matches should be near zero. If they aren't, the machine lied to us!
    log::debug!("Identical rolls at matching positions: {}/{}", matches, sample_size);
    assert!(
        matches < (sample_size / 10), 
        "Sequences are too similar! We have a determinist amongst us!"
    );
    
    assert_ne!(seq_a, seq_b, "Something gone really, really wrong - seq B perfectly mirrored A!");
}

#[test]
fn longest_streak_above_50() {
    let mut longest = 0;
    let mut curr = 0;
    for _ in 0..10_000_000 {
        let x = 1.d100();
        if x > 50 {
            curr += 1;
            if curr > longest {
                longest = curr;
            }
        } else {
            curr = 0;
        }
    }
    _ = env_logger::try_init();
    assert!(longest <= 24, "Expected 24 at most, got {}", longest);
    log::debug!("Longest streak above 50: {longest}");
}

#[test]
fn appearance_of_1_to_100() {
    let mut app: HashMap<usize, usize, BuildNoHashHasher<usize>> = HashMap::default();
    for _ in 0..SPAM_THRESHOLD * 100 {
        let x = 1.d100();
        *app.entry(x).or_default() += 1;
    }
    _ = env_logger::try_init();
    for (k,c) in app {
        log::debug!("{k} → {c} times out of 1,000,000");
    }
}

#[test]
fn float_dice_bounds_f32() {
    // Test standard integer float casting (e.g., 2.0 d6 should be between 2.0 and 12.0)
    for _ in 0..SPAM_THRESHOLD {
    let roll: f32 = 2.0.d6();
    assert!(roll >= 2.0 && roll <= 12.0, "Roll out of bounds: {}", roll);

    // Test pure fractional roll (e.g., 0.5 d10 should be between 0.0 and 5.0)
    let fractional_roll: f32 = 0.5.d10();
    assert!(fractional_roll >= 0.0 && fractional_roll <= 5.0, "Fractional roll out of bounds: {}", fractional_roll);

    // Test mixed chop-suey roll (1.25 d10 -> max could be 10 + 2.5 = 12.5)
    let chop_roll: f32 = 1.25.d10();
    assert!(chop_roll >= 1.0 && chop_roll <= 12.5, "Chop-suey roll out of bounds: {}", chop_roll);
    }
}

#[test]
fn float_dice_bounds_f64() {
    for _ in 0..SPAM_THRESHOLD {
    let roll: f64 = 3.0.d20();
    assert!(roll >= 3.0 && roll <= 60.0);

    let sub_one: f64 = 0.75.d4();
    assert!(sub_one >= 0.0 && sub_one <= 3.0);
    }
}

#[test]
fn float_zero_and_edge_cases() {
    for _ in 0..SPAM_THRESHOLD {
    // Rolling zero dice should yield zero
    let zero_roll: f32 = 0.0.d100();
    assert_eq!(zero_roll, 0.0);
    }
}
