use dicebag::*;

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
