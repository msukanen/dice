# Dicebag

This library contains various "dice rolling" functions and traits.

## dicebag::DiceExt

Dice rolling is currently implemented for most basic numeric types from i8 to i128, u8 to u128, and usize.

## Usage

```rust
use dicebag::DiceExt;
// Some normie dice:
let a = 3.d6();   // 3d6
let b = 10.d10(); // 10d10

// Delta variancy:
let d = 1.0.delta(5);
    //→ 1.0 ± 5%
    //→ 0.95 .. 1.05

// 50% chance of `c` becoming 5 (otherwise 0).
let c = chance_of!(5, 50);
```
