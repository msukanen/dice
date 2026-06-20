# Dicebag — Kawaii Chaos, Industrial-Grade Randomness

This library contains various "dice rolling" functions and traits.

## Primitives? Yes, please

In most cases, anything from `i8`/`u8` up to `i128`/`u128`
and `usize` is supported (alongside `f32`/`f64` for a few
functions).

## dicebag::DiceExt

Covers the core intent dice rollings, e.g. `3.d6()`, `2.d10()`.

```rust
use dicebag::DiceExt;
let a = 3.d6();
let b = 2_u8.d4();
let c = 5.d(a-3); // FYI: zero as dice size results in 0...
```

## dicebag::HiLo

Coin flipping, using whatever datatype you implement it for…
Comes with two convenience macros so that you don't need to
write those yourself:

```rust
use dicebag::{DiceExt, HiLo, lo, hi};
if lo!() {/* do something if result was "low" */}
if hi!() {/* do something if result was "high" */}
```

## dicebag::InclusiveRandomRange

To get a random value within given range.

```rust
use dicebag::InclusiveRandomRange;
let range = 6..=12;
let v = range.random_of();
```

## dicebag::RandomOf\<T\>

A trait to get some random entry of e.g. `Vec` or `HashSet`.

Just make sure your container has at least one entry in it as otherwise
things will catch fire (panic). `.random_of()` really can't choose
a random element out of nothing given…

## dicebag::KeyedRandomOf\<K, T\>

A trait for randomizing against e.g. `HashMap` entries.
Just like with `dicebag::RandomOf`, panic ensues if the map is empty.

```rust
use dicebag::RandomOf;
let v = vec![2,4,6,8,10];
let x: i32 = v.random_of();

#[derive(Clone)]
struct Abc { tag: String };
let abc = vec![Abc{tag:"a".into()}, Abc{tag:"b".into()}, Abc{tag:"c".into()}];
let x = abc.random_of();
assert!(x.tag == "a" || x.tag == "b" || x.tag == "c");
```

## Technobabble

***dicebag*** isn't a PRNG, and it's not CRNG either. It's **Chaos-RNG**,
a one-way entropy shredder.

By design, it:

* does **not** produce reproducible sequences
* does **not** preserve information
* does **not** support seeding (nor could it, even if tried)
* cannot be reversed
* cannot be predicted
* cannot be modeled
* cannot be replayed

…ergo, if you attempt to use ***dicebag*** for encoding, encryption,
obfuscation, or *anything* requiring deterministic output, please don't —
the result will be unrecoverable, unrepeatabe, and basically un-anything-able.

```text
input →  (っ◔◡◔)っ  ~{ swirl }~
                     ↳ output dissolved

         .-------.
        /  CHAOS  \
       |  RNG CORE |
        \  _____  /
         |/     \|
    input         out ~~> [noise]
                   \
                    \__ unrecoverable
```
