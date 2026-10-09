[![Build](https://github.com/rust-bitcoin/rust-ordered/actions/workflows/rust.yml/badge.svg)](https://github.com/rust-bitcoin/rust-ordered/actions)

Arbitrary Ordering
==================

Provides a wrapper for types that can technically implement `PartialOrd`/`Ord`
but for semantic reasons it is nonsensical.

## Examples

You might want to use a type as a key in a `BTreeMap`, this requires `Ord` even though it might be
nonsensical to implement a total order for the type.

```rust
use std::cmp::Ordering;
use std::collections::BTreeMap;
use ordered::{ArbitraryOrd, Ordered};

/// A Foo type.
///
/// We do not want users to be able to write `a < b` because it is meaningless
/// to compare the two but we wish to use `Foo`, for example, as a `BTreeMap` key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Foo {
    /// A space foo.
    Space(u32),
    /// A time foo.
    Time(u32)
}

impl ArbitraryOrd for Foo {
    fn arbitrary_cmp(&self, other: &Self) -> Ordering {
        use Foo::*;
        match (self, other) {
            (Space(_), Time(_)) => Ordering::Less,
            (Time(_), Space(_)) => Ordering::Greater,
            (Space(this), Space(that)) => this.cmp(that),
            (Time(this), Time(that)) => this.cmp(that),
        }
    }
}

let a = Foo::Space(50);
let b = Foo::Time(50);

let mut map = BTreeMap::new();

// error[E0277]: the trait bound `Foo: Ord` is not satisfied
// map.insert(a, "some interesting value");

map.insert(Ordered(a), "some interesting value");
map.insert(Ordered(b), "some other interesting value");
```

Perhaps you would like to derive `Ord` on a complex type that contains a type that does not
implement `Ord`.

```rust
/// An example struct that contains a `Foo` but derives `PartialOrd` and `Ord`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Adt {
    x: u32,
    p: Ordered<Foo>,
}

// Then there are various ways to get at the inner data.

let adt = Adt { x: 42, p: Foo::Space(50).into() };

println!("We can explicitly deref: {}", *adt.p);
println!("Or use deref coercion: {}", adt.p);
println!("Or we can use borrow: {}", &adt.p);
```

### Floats

`f32` and `f64` implement `ArbitraryOrd`, so `Ordered<f64>` implements `Eq` and `Ord` and can be
used as a `BTreeMap` key. Finite values and infinities are ordered numerically, `-0.0` and `0.0`
are equal, and `NaN` is greater than every other value. All `NaN`s are equal to each other,
regardless of sign or payload, so `Ordered(f64::NAN) == Ordered(f64::NAN)` even though
`f64::NAN != f64::NAN`.

```rust
use std::collections::BTreeMap;
use ordered::Ordered;

let mut map = BTreeMap::new();
map.insert(Ordered(1.5_f64), "finite");
map.insert(Ordered(f64::NAN), "nan");

assert_eq!(map.get(Ordered::from_ref(&f64::NAN)), Some(&"nan"));
```

## Minimum Supported Rust Version (MSRV)

This library should compile with any combination of features on **Rust 1.63.0**.

Use `Cargo-minimal.lock` to build the MSRV by copying to `Cargo.lock` and building.

## Licensing

The code in this project is licensed under the [Creative Commons CC0 1.0 Universal license](LICENSE).
We use the [SPDX license list](https://spdx.org/licenses/) and [SPDX IDs](https://spdx.dev/ids/).
