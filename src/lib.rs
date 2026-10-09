// SPDX-License-Identifier: CC0-1.0

//! Provides a wrapper for types that can technically implement `PartialOrd`/`Ord` but for semantic
//! reasons it is nonsensical.
//!
//! `PartialOrd` and `Ord` are often useful and/or required. For example, [`Ordered`] allows one to
//! use such a type as a key in a `BTreeMap` (which requires ordered keys).
//!
//! For a full example see [`examples/point.rs`].
//!
//! # Examples
//!
//! ```
//! # #![allow(unused)] // Because of `Adt`.
//! use core::cmp::Ordering;
//! use ordered::{ArbitraryOrd, Ordered};
//!
//! /// A point in 2D space.
//! ///
//! /// We do not want users to be able to write `a < b` because it is not well defined.
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! struct Point {
//!     x: u32,
//!     y: u32,
//! }
//!
//! impl ArbitraryOrd for Point {
//!     fn arbitrary_cmp(&self, other: &Self) -> Ordering {
//!         // Just use whatever order tuple cmp gives us.
//!         (self.x, self.y).cmp(&(other.x, other.y))
//!     }
//! }
//!
//! /// `Ordered` allows users to derive `PartialOrd` on types that include a `Point`.
//! #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
//! struct Adt {
//!     name: String,
//!     point: Ordered<Point>,
//! }
//! ```
//!
//! [`examples/point.rs`]: <https://github.com/rust-bitcoin/rust-ordered/blob/master/examples/point.rs>

#![no_std]
// Experimental features we need.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// Coding conventions.
#![warn(missing_docs)]
#![warn(deprecated_in_future)]
#![doc(test(attr(warn(unused))))]

use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut};

/// Trait for types that perform an arbitrary ordering.
///
/// More specifically, this trait is for types that can be given a total order
/// but for which that order is semantically nonsensical.
///
/// # Requirements
///
/// [`Ordered`] implements `PartialEq`, `Eq`, `PartialOrd` and `Ord` entirely in terms of
/// `arbitrary_cmp`, so `arbitrary_cmp` must be a total order: reflexive, antisymmetric, transitive
/// and total.
/// If `T: Hash` then any two values for which `arbitrary_cmp` returns `Ordering::Equal` must also
/// hash equally. Violating either requirement is not undefined behaviour, but collections such as
/// `BTreeMap` and `HashMap` will behave incorrectly.
///
/// `arbitrary_cmp` does not have to agree with `T`'s own `PartialEq`, and `Ordered` never calls
/// it. For example `f64::NAN != f64::NAN`, but `Ordered(f64::NAN) == Ordered(f64::NAN)`.
///
/// # Examples
///
/// ```
/// # #![allow(unused)] // Because of `Adt`.
/// use core::cmp::Ordering;
/// use ordered::ArbitraryOrd;
///
/// /// A point in 2D space.
/// ///
/// /// We do not want users to be able to write `a < b` because it is not well defined.
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// struct Point {
///     x: u32,
///     y: u32,
/// }
///
/// impl ArbitraryOrd for Point {
///     fn arbitrary_cmp(&self, other: &Self) -> Ordering {
///         // Just use whatever order tuple cmp gives us.
///         (self.x, self.y).cmp(&(other.x, other.y))
///     }
/// }
/// ```
pub trait ArbitraryOrd<Rhs = Self> {
    /// Implements a meaningless, arbitrary ordering.
    fn arbitrary_cmp(&self, other: &Rhs) -> Ordering;
}

/// A wrapper type that implements `PartialEq`, `Eq`, `PartialOrd` and `Ord` using
/// [`ArbitraryOrd::arbitrary_cmp`].
///
/// `Hash` hashes the inner `T`. The `Borrow<T>` impl lets a map keyed by `Ordered<T>` be queried
/// with a `&T`, but such a lookup compares and hashes with `T`'s own `Ord`, `Eq` and `Hash`, not
/// `arbitrary_cmp`. It does not compile when `T` lacks those traits, as with `f64` in a
/// `BTreeMap`, and it only finds the right entry when they agree with `arbitrary_cmp`. Query with
/// [`Ordered::from_ref`] instead.
///
/// Because equality comes from `arbitrary_cmp`, `Ordered<T>` does not implement `PartialEq`
/// unless `T: ArbitraryOrd`:
///
/// ```compile_fail,E0369
/// use ordered::Ordered;
///
/// let _ = Ordered(String::new()) == Ordered(String::new());
/// ```
///
/// # Examples
///
/// ```
/// # #![allow(unused)] // Because of `Adt`.
/// use core::cmp::Ordering;
/// use ordered::{ArbitraryOrd, Ordered};
///
/// /// A point in 2D space.
/// ///
/// /// We do not want users to be able to write `a < b` because it is not well defined.
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// struct Point {
///     x: u32,
///     y: u32,
/// }
///
/// impl ArbitraryOrd for Point {
///     fn arbitrary_cmp(&self, other: &Self) -> Ordering {
///         // Just use whatever order tuple cmp gives us.
///         (self.x, self.y).cmp(&(other.x, other.y))
///     }
/// }
///
/// let point = Point { x: 0, y: 1 };
/// let ordered = Ordered(point);
///
/// assert_eq!(*ordered, point); // Use `ops::Deref`.
/// assert_eq!(&ordered.0, ordered.as_ref()); // Use the public inner field or `AsRef`.
/// ```
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Ordered<T>(pub T);

impl<T: Copy> Copy for Ordered<T> {}

impl<T> Ordered<T> {
    /// Creates a new wrapped ordered type.
    ///
    /// The inner type is public so this function is never explicitly needed.
    pub const fn new(inner: T) -> Self { Self(inner) }

    /// Creates an `Ordered<T>` from a reference.
    ///
    /// This allows: `let found = map.get(Ordered::from_ref(&a));`
    #[allow(clippy::ptr_as_ptr)]
    pub fn from_ref(value: &T) -> &Self { unsafe { &*(value as *const _ as *const Self) } }

    /// Returns a reference to the inner object.
    ///
    /// We also implement [`core::borrow::Borrow`] so this function is never explicitly needed.
    #[deprecated(since = "0.3.0", note = "use `ops::Deref` instead")]
    pub const fn as_inner(&self) -> &T { &self.0 }

    /// Returns the inner object.
    ///
    /// We also implement [`core::ops::Deref`] so this function is never explicitly needed.
    #[deprecated(since = "0.3.0", note = "use `ops::Deref` instead")]
    pub fn into_inner(self) -> T { self.0 }
}

impl<T: ArbitraryOrd> ArbitraryOrd for &T {
    fn arbitrary_cmp(&self, other: &Self) -> Ordering { (*self).arbitrary_cmp(other) }
}

macro_rules! impl_arbitrary_ord_for_float {
    ($($t:ty),*) => {
        $(
            #[doc = concat!(
                "Orders `", stringify!($t), "` values numerically, treats `-0.0` and `0.0` as ",
                "equal, treats all `NaN`s as equal regardless of sign or payload, and orders ",
                "`NaN` above every other value.\n\n",
                "This differs from `", stringify!($t), "::total_cmp`, which distinguishes `-0.0` ",
                "from `0.0` and orders `NaN`s by sign and payload.\n\n",
                "# Examples\n\n",
                "```\n",
                "use std::collections::BTreeMap;\n",
                "use ordered::Ordered;\n\n",
                "let mut map = BTreeMap::new();\n",
                "map.insert(Ordered(1.5_", stringify!($t), "), \"finite\");\n",
                "map.insert(Ordered(", stringify!($t), "::NAN), \"nan\");\n\n",
                "assert_eq!(map.get(Ordered::from_ref(&", stringify!($t), "::NAN)), ",
                "Some(&\"nan\"));\n",
                "```"
            )]
            impl ArbitraryOrd for $t {
                fn arbitrary_cmp(&self, other: &Self) -> Ordering {
                    self.partial_cmp(other)
                        .unwrap_or_else(|| self.is_nan().cmp(&other.is_nan()))
                }
            }
        )*
    };
}
impl_arbitrary_ord_for_float!(f32, f64);

impl<T: ArbitraryOrd> PartialEq for Ordered<T> {
    fn eq(&self, other: &Self) -> bool { self.0.arbitrary_cmp(&other.0) == Ordering::Equal }
}

impl<T: ArbitraryOrd> Eq for Ordered<T> {}

impl<T: ArbitraryOrd> PartialOrd for Ordered<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl<T: ArbitraryOrd> Ord for Ordered<T> {
    fn cmp(&self, other: &Self) -> Ordering { self.0.arbitrary_cmp(&other.0) }
}

impl<T: Hash> Hash for Ordered<T> {
    fn hash<H: Hasher>(&self, state: &mut H) { self.0.hash(state) }
}

impl<T: fmt::Display> fmt::Display for Ordered<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}

impl<T> From<T> for Ordered<T> {
    fn from(inner: T) -> Self { Self(inner) }
}

impl<T> AsRef<T> for Ordered<T> {
    fn as_ref(&self) -> &T { &self.0 }
}

impl<T> AsMut<T> for Ordered<T> {
    fn as_mut(&mut self) -> &mut T { &mut self.0 }
}

impl<T> Borrow<T> for Ordered<T> {
    fn borrow(&self) -> &T { &self.0 }
}

impl<T> BorrowMut<T> for Ordered<T> {
    fn borrow_mut(&mut self) -> &mut T { &mut self.0 }
}

impl<T> Deref for Ordered<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> DerefMut for Ordered<T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct Point {
        x: u32,
        y: u32,
    }

    impl Point {
        fn new(x: u32, y: u32) -> Self { Point { x, y } }
    }

    impl ArbitraryOrd for Point {
        fn arbitrary_cmp(&self, other: &Self) -> Ordering {
            (self.x, self.y).cmp(&(other.x, other.y))
        }
    }

    #[test]
    fn can_compare() {
        let a = Point::new(2, 3);
        let b = Point::new(5, 7);

        assert!(Ordered(a) < Ordered(b));
    }

    #[test]
    fn can_compare_with_from_ref() {
        let a = Point::new(2, 3);
        let b = Point::new(5, 7);

        assert!(Ordered::from_ref(&a) < Ordered::from_ref(&b));
    }

    #[test]
    fn can_compare_with_reference() {
        let a = Point::new(2, 3);
        let b = Point::new(5, 7);

        assert!(Ordered(&a) < Ordered(&b));
    }

    // Copied from https://rust-lang.github.io/api-guidelines/interoperability.html#c-send-sync
    #[test]
    fn send() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        struct Point {
            x: u32,
            y: u32,
        }

        impl ArbitraryOrd for Point {
            fn arbitrary_cmp(&self, other: &Self) -> Ordering {
                (self.x, self.y).cmp(&(other.x, other.y))
            }
        }

        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Ordered<Point>>();
        assert_sync::<Ordered<Point>>();
    }

    #[test]
    fn trait_is_object_safe() {
        extern crate std;
        use std::boxed::Box;

        // If this test builds then `ArbitraryOrd` is object safe.
        #[allow(dead_code)]
        struct ObjectSafe {
            p: Box<dyn ArbitraryOrd<Self>>,
            q: Box<dyn PartialOrd<Self>>, // Sanity check.
        }
    }

    #[test]
    fn f64_sorts_across_every_category_as_a_total_order() {
        extern crate std;
        use std::vec::Vec;

        let payload_nan = f64::from_bits(0x7ff8_0000_0000_0001);
        let negative_signalling_nan = f64::from_bits(0xfff0_0000_0000_0001);
        assert!(payload_nan.is_nan() && negative_signalling_nan.is_nan());
        assert_ne!(payload_nan.to_bits(), f64::NAN.to_bits());

        let mut values: Vec<Ordered<f64>> = [
            f64::NAN,
            f64::INFINITY,
            -0.0,
            payload_nan,
            0.0,
            -f64::NAN,
            f64::MAX,
            negative_signalling_nan,
            f64::NEG_INFINITY,
            -1.0,
            1.0,
        ]
        .into_iter()
        .map(Ordered)
        .collect();
        values.sort();

        let (non_nan, nans) = values.split_at(7);
        let expected = [f64::NEG_INFINITY, -1.0, 0.0, 0.0, 1.0, f64::MAX, f64::INFINITY];
        for (actual, expected) in non_nan.iter().zip(expected) {
            assert_eq!(*actual, Ordered(expected));
        }
        assert_eq!(nans.len(), 4);
        for a in nans {
            for b in nans {
                assert_eq!(a.cmp(b), Ordering::Equal);
            }
        }

        for a in &values {
            for b in &values {
                assert_eq!(a.cmp(b), b.cmp(a).reverse());
                assert_eq!(a.partial_cmp(b), Some(a.cmp(b)));
                assert_eq!(a == b, a.cmp(b) == Ordering::Equal);
                for c in &values {
                    if a <= b && b <= c {
                        assert!(a <= c);
                    }
                }
            }
        }
    }

    #[test]
    fn f32_orders_like_f64() {
        let payload_nan = f32::from_bits(0x7fc0_0001);
        assert!(payload_nan.is_nan());

        assert!(Ordered(1.0_f32) < Ordered(2.0_f32));
        assert!(Ordered(-0.0_f32) == Ordered(0.0_f32));
        assert!(Ordered(f32::INFINITY) < Ordered(f32::NAN));
        assert!(Ordered(f32::NEG_INFINITY) < Ordered(f32::NAN));
        assert!(Ordered(f32::NAN) == Ordered(-f32::NAN));
        assert!(Ordered(f32::NAN) == Ordered(payload_nan));
    }

    #[test]
    fn ordered_float_reference_uses_float_ordering() {
        let (one, two, nan) = (1.0_f64, 2.0_f64, f64::NAN);

        assert!(Ordered(&one) < Ordered(&two));
        assert!(Ordered(&two) < Ordered(&nan));
        assert!(Ordered(&nan) == Ordered(&nan));
    }

    #[test]
    fn btree_map_collapses_equal_floats_into_one_key() {
        extern crate std;
        use std::collections::BTreeMap;

        let mut map = BTreeMap::new();
        map.insert(Ordered(0.0_f64), "positive zero");
        map.insert(Ordered(-0.0_f64), "negative zero");
        map.insert(Ordered(f64::NAN), "nan");
        map.insert(Ordered(-f64::NAN), "negative nan");

        assert_eq!(map.len(), 2);
        assert_eq!(map.get(Ordered::from_ref(&0.0_f64)), Some(&"negative zero"));
        assert_eq!(map.get(Ordered::from_ref(&f64::NAN)), Some(&"negative nan"));
    }
}
