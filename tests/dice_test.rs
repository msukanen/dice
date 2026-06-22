use std::{collections::HashSet, thread};

use dicebag::*;
use serde::Deserialize;

/// See that D6 rolls stay within range.
#[test]
fn d6_stay_in_range() {
    for _ in 0..10_000 {
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

#[test]
fn chance_macro_works() {
    for _ in 0..20 {
        println!("{}", percentage_chance_of!(5, 50))
    }
}

#[test]
fn random_of_vec() {
    let vs = vec![&1,&2,&3,&4,&5];
    for _ in 0..10_000 {
        let v = vs.random_of();
        assert!(*v >= 1 && *v <= 5, "Hol' a moment! We got an out of bounds, wild {v} amongst us!");
    }
}

#[test]
fn random_of_f64() {
    let vs = 0.5..=2.0;
    for _ in 0..100_001 {
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
    for _ in 0..10_000 {
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
    for _ in 0..100_000 {
        if c.roll() == 1 {
            ones += 1;
        }
    }
    assert!(ones >= 45000 && ones <= 55000, "Strange number of ones: {ones}; expected 4.5–5.5 mid range");
    _ = env_logger::try_init();
    log::debug!("Exactly {ones} '1's out of 100,000 pool of 50% chances.")
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
fn test_dicebag_is_completely_non_deterministic() {
    use std::{time::Duration, thread};
    
    _ = env_logger::try_init();
    // seq of 10,000 dice rolls (with d10000 for high variancy)
    let sample_size = 10_000;
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
