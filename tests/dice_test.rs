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
        log::debug!("r = {r}");
        if old_roll == r {
            repeats += 1;
        }
        old_roll = r;
        assert!(r >= 3 && r <= 12, "Roll of {r} is out of bounds of [3..=12]!");
    }

    log::debug!("Repeats: {repeats} out of 10,000")
}
