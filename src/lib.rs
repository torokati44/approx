// Copyright 2015 Brendan Zabarauskas
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A crate that provides facilities for testing the approximate equality of floating-point
//! based types, using either relative difference, or units in the last place (ULPs)
//! comparisons.
//!
//! You can also use the `approx_{eq, ne}!` `assert_approx_{eq, ne}!` macros to test for equality
//! using a more positional style.
//!
//! ```rust
//! #[macro_use]
//! extern crate approx;
//! use std::f64;
//!
//! # fn main() {
//! relative_eq!(1.0, 1.0);
//! relative_eq!(1.0, 1.0, epsilon = f64::EPSILON);
//! relative_eq!(1.0, 1.0, max_relative = 1.0);
//! relative_eq!(1.0, 1.0, epsilon = f64::EPSILON, max_relative = 1.0);
//! relative_eq!(1.0, 1.0, max_relative = 1.0, epsilon = f64::EPSILON);
//!
//! ulps_eq!(1.0, 1.0);
//! ulps_eq!(1.0, 1.0, epsilon = f64::EPSILON);
//! ulps_eq!(1.0, 1.0, max_ulps = 4);
//! ulps_eq!(1.0, 1.0, epsilon = f64::EPSILON, max_ulps = 4);
//! ulps_eq!(1.0, 1.0, max_ulps = 4, epsilon = f64::EPSILON);
//! # }
//! ```
//!
//! ## Implementing approximate equality for custom types
//!
//! The `ApproxEq` trait allows approximate equalities to be implemented on types, based on the
//! fundamental floating point implementations.
//!
//! For example, we might want to be able to do approximate assertions on a complex number type:
//!
//! ```rust
//! #[macro_use]
//! extern crate approx;
//! # use approx::ApproxEq;
//! #[derive(Debug)]
//! struct Complex<T> {
//!     x: T,
//!     i: T,
//! }
//! # impl<T: ApproxEq> ApproxEq for Complex<T> where T::Epsilon: Copy {
//! #     type Epsilon = T::Epsilon;
//! #     fn default_epsilon() -> T::Epsilon { T::default_epsilon() }
//! #     fn default_max_relative() -> T::Epsilon { T::default_max_relative() }
//! #     fn default_max_ulps() -> u32 { T::default_max_ulps() }
//! #     fn relative_eq(&self, other: &Self, epsilon: T::Epsilon, max_relative: T::Epsilon) -> bool { T::relative_eq(&self.x, &other.x, epsilon, max_relative) && T::relative_eq(&self.i, &other.i, epsilon, max_relative) }
//! #     fn ulps_eq(&self, other: &Self, epsilon: T::Epsilon, max_ulps: u32) -> bool { T::ulps_eq(&self.x, &other.x, epsilon, max_ulps) && T::ulps_eq(&self.i, &other.i, epsilon, max_ulps) }
//! # }
//!
//! # fn main() {
//! let x = Complex { x: 1.2, i: 2.3 };
//!
//! assert_relative_eq!(x, x);
//! assert_ulps_eq!(x, x, max_ulps = 4);
//! # }
//! ```
//!
//! To do this we can implement `ApproxEq` generically in terms of a type parameter that also
//! implements `ApproxEq`. This means that we can make comparisons for either `Complex<f32>` or
//! `Complex<f64>`:
//!
//! ```rust
//! # use approx::ApproxEq;
//! # #[derive(Debug)]
//! # struct Complex<T> { x: T, i: T, }
//! #
//! impl<T: ApproxEq> ApproxEq for Complex<T> where
//!     T::Epsilon: Copy,
//! {
//!     type Epsilon = T::Epsilon;
//!
//!     fn default_epsilon() -> T::Epsilon {
//!         T::default_epsilon()
//!     }
//!
//!     fn default_max_relative() -> T::Epsilon {
//!         T::default_max_relative()
//!     }
//!
//!     fn default_max_ulps() -> u32 {
//!         T::default_max_ulps()
//!     }
//!
//!     fn relative_eq(&self, other: &Self, epsilon: T::Epsilon, max_relative: T::Epsilon) -> bool {
//!         T::relative_eq(&self.x, &other.x, epsilon, max_relative) &&
//!         T::relative_eq(&self.i, &other.i, epsilon, max_relative)
//!     }
//!
//!     fn ulps_eq(&self, other: &Self, epsilon: T::Epsilon, max_ulps: u32) -> bool {
//!         T::ulps_eq(&self.x, &other.x, epsilon, max_ulps) &&
//!         T::ulps_eq(&self.i, &other.i, epsilon, max_ulps)
//!     }
//! }
//! ```
//!
//! # References
//!
//! Floating point is hard! Thanks goes to these links for helping to make things a _little_
//! easier to understand:
//!
//! - [Comparing Floating Point Numbers, 2012 Edition]
//!   (https://randomascii.wordpress.com/2012/02/25/comparing-floating-point-numbers-2012-edition/)
//! - [The Floating Point Guide - Comparison](http://floating-point-gui.de/errors/comparison/)
//! - [What Every Computer Scientist Should Know About Floating-Point Arithmetic]
//!   (https://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html)

#![cfg_attr(feature="no_std", no_std)]
#![cfg_attr(feature="no_std", feature(core_float))]
#[cfg(feature="no_std")]
use core as std;

#[cfg(feature="no_std")]
#[allow(unused_imports)] // Get a warning otherwise. This seems like a bug.
use core::num::Float;

mod macros;

/// Equality comparisons based on floating point tolerances.
pub trait ApproxEq: Sized {
    /// Used for specifying relative comparisons.
    type Epsilon;

    /// The default tolerance to use when testing values that are close together.
    ///
    /// This is used when no `epsilon` value is supplied to the `relative_eq` or `ulps_eq` macros.
    fn default_epsilon() -> Self::Epsilon;

    /// The default relative tolerance for testing values that are far-apart.
    ///
    /// This is used when no `max_relative` value is supplied to the `relative_eq` macro.
    fn default_max_relative() -> Self::Epsilon;

    /// The default ULPs to tolerate when testing values that are far-apart.
    ///
    /// This is used when no `max_relative` value is supplied to the `relative_eq` macro.
    fn default_max_ulps() -> u32;

    /// A test for equality that uses a relative comparison if the values are far apart.
    fn relative_eq(&self,
                   other: &Self,
                   epsilon: Self::Epsilon,
                   max_relative: Self::Epsilon)
                   -> bool;

    /// The inverse of `ApproxEq::relative_eq`.
    fn relative_ne(&self,
                   other: &Self,
                   epsilon: Self::Epsilon,
                   max_relative: Self::Epsilon)
                   -> bool {
        !Self::relative_eq(self, other, epsilon, max_relative)
    }

    /// A test for equality that uses units in the last place (ULP) if the values are far apart.
    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool;

    /// The inverse of `ApproxEq::ulps_eq`.
    fn ulps_ne(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        !Self::ulps_eq(self, other, epsilon, max_ulps)
    }
}

macro_rules! impl_float_approx_eq {
    ($T:ident, $U:ident) => {
        impl ApproxEq for $T {
            type Epsilon = $T;

            #[inline]
            fn default_epsilon() -> $T { std::$T::EPSILON }

            #[inline]
            fn default_max_relative() -> $T { std::$T::EPSILON }

            #[inline]
            fn default_max_ulps() -> u32 { 4 }

            #[inline]
            fn relative_eq(&self, other: &$T, epsilon: $T, max_relative: $T) -> bool {
                // Implementation based on: [Comparing Floating Point Numbers, 2012 Edition]
                // (https://randomascii.wordpress.com/2012/02/25/comparing-floating-point-numbers-2012-edition/)

                // Handle same infinities
                if self == other {
                    return true;
                }

                let abs_diff = $T::abs(self - other);

                // For when the numbers are really close together
                if abs_diff <= epsilon {
                    return true;
                }

                let abs_self = $T::abs(*self);
                let abs_other = $T::abs(*other);

                // Handle oppsite infinities
                if abs_self == abs_other && abs_diff == abs_self {
                    return false;
                }

                let largest = if abs_other > abs_self { abs_other } else { abs_self };

                // Use a relative difference comparison
                abs_diff <= largest * max_relative
            }

            #[inline]
            fn ulps_eq(&self, other: &$T, epsilon: $T, max_ulps: u32) -> bool {
                // Implementation based on: [Comparing Floating Point Numbers, 2012 Edition]
                // (https://randomascii.wordpress.com/2012/02/25/comparing-floating-point-numbers-2012-edition/)

                let abs_diff = $T::abs(self - other);

                // For when the numbers are really close together
                if abs_diff <= epsilon {
                    return true;
                }

                // Trivial negative sign check
                if self.signum() != other.signum() {
                    return false;
                }

                // ULPS difference comparison
                let int_self: $U = unsafe { std::mem::transmute(*self) };
                let int_other: $U = unsafe { std::mem::transmute(*other) };

                $U::abs(int_self - int_other) < max_ulps as $U
            }
        }
    }
}

impl_float_approx_eq!(f32, i32);
impl_float_approx_eq!(f64, i64);


impl<'a, T: ApproxEq> ApproxEq for &'a T {
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    #[inline]
    fn relative_eq(&self, other: &&'a T, epsilon: T::Epsilon, max_relative: T::Epsilon) -> bool {
        T::relative_eq(*self, *other, epsilon, max_relative)
    }

    #[inline]
    fn ulps_eq(&self, other: &&'a T, epsilon: T::Epsilon, max_ulps: u32) -> bool {
        T::ulps_eq(*self, *other, epsilon, max_ulps)
    }
}

impl<'a, T: ApproxEq> ApproxEq for &'a mut T {
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    #[inline]
    fn relative_eq(&self,
                   other: &&'a mut T,
                   epsilon: T::Epsilon,
                   max_relative: T::Epsilon)
                   -> bool {
        T::relative_eq(*self, *other, epsilon, max_relative)
    }

    #[inline]
    fn ulps_eq(&self, other: &&'a mut T, epsilon: T::Epsilon, max_ulps: u32) -> bool {
        T::ulps_eq(*self, *other, epsilon, max_ulps)
    }
}

/// The requisite parameters for testing for approximate equality using a
/// relative based comparison.
///
/// This is not normally used directly, rather via the
/// `assert_relative_{eq|ne}!` and `relative_{eq|ne}!` macros.
///
/// # Example
///
/// ```rust
/// use std::f64;
/// use approx::Relative;
///
/// Relative::default().eq(&1.0, &1.0);
/// Relative::default().epsilon(f64::EPSILON).eq(&1.0, &1.0);
/// Relative::default().max_relative(1.0).eq(&1.0, &1.0);
/// Relative::default().epsilon(f64::EPSILON).max_relative(1.0).eq(&1.0, &1.0);
/// Relative::default().max_relative(1.0).epsilon(f64::EPSILON).eq(&1.0, &1.0);
/// ```
pub struct Relative<T: ApproxEq> {
    /// The tolerance to use when testing values that are close together.
    pub epsilon: T::Epsilon,
    /// The relative tolerance for testing values that are far-apart.
    pub max_relative: T::Epsilon,
}

impl<T> Default for Relative<T>
    where T: ApproxEq
{
    #[inline]
    fn default() -> Relative<T> {
        Relative {
            epsilon: T::default_epsilon(),
            max_relative: T::default_max_relative(),
        }
    }
}

impl<T> Relative<T>
    where T: ApproxEq
{
    /// Replace the epsilon value with the one specified.
    #[inline]
    pub fn epsilon(self, epsilon: T::Epsilon) -> Relative<T> {
        Relative {
            epsilon: epsilon,
            ..self
        }
    }

    /// Replace the maximum relative value with the one specified.
    #[inline]
    pub fn max_relative(self, max_relative: T::Epsilon) -> Relative<T> {
        Relative {
            max_relative: max_relative,
            ..self
        }
    }

    /// Peform the equality comparison
    #[inline]
    pub fn eq(self, lhs: &T, rhs: &T) -> bool {
        T::relative_eq(lhs, rhs, self.epsilon, self.max_relative)
    }

    /// Peform the inequality comparison
    #[inline]
    pub fn ne(self, lhs: &T, rhs: &T) -> bool {
        T::relative_ne(lhs, rhs, self.epsilon, self.max_relative)
    }
}

/// The requisite parameters for testing for approximate equality using an ULPs
/// based comparison.
///
/// This is not normally used directly, rather via the `assert_ulps_{eq|ne}!`
/// and `ulps_{eq|ne}!` macros.
///
/// # Example
///
/// ```rust
/// use std::f64;
/// use approx::Ulps;
///
/// Ulps::default().eq(&1.0, &1.0);
/// Ulps::default().epsilon(f64::EPSILON).eq(&1.0, &1.0);
/// Ulps::default().max_ulps(4).eq(&1.0, &1.0);
/// Ulps::default().epsilon(f64::EPSILON).max_ulps(4).eq(&1.0, &1.0);
/// Ulps::default().max_ulps(4).epsilon(f64::EPSILON).eq(&1.0, &1.0);
/// ```
pub struct Ulps<T: ApproxEq> {
    /// The tolerance to use when testing values that are close together.
    pub epsilon: T::Epsilon,
    /// The ULPs to tolerate when testing values that are far-apart.
    pub max_ulps: u32,
}

impl<T> Default for Ulps<T>
    where T: ApproxEq
{
    #[inline]
    fn default() -> Ulps<T> {
        Ulps {
            epsilon: T::default_epsilon(),
            max_ulps: T::default_max_ulps(),
        }
    }
}

impl<T> Ulps<T>
    where T: ApproxEq
{
    /// Replace the epsilon value with the one specified.
    #[inline]
    pub fn epsilon(self, epsilon: T::Epsilon) -> Ulps<T> {
        Ulps {
            epsilon: epsilon,
            ..self
        }
    }

    /// Replace the max ulps value with the one specified.
    #[inline]
    pub fn max_ulps(self, max_ulps: u32) -> Ulps<T> {
        Ulps {
            max_ulps: max_ulps,
            ..self
        }
    }

    /// Peform the equality comparison
    #[inline]
    pub fn eq(self, lhs: &T, rhs: &T) -> bool {
        T::ulps_eq(lhs, rhs, self.epsilon, self.max_ulps)
    }

    /// Peform the inequality comparison
    #[inline]
    pub fn ne(self, lhs: &T, rhs: &T) -> bool {
        T::ulps_ne(lhs, rhs, self.epsilon, self.max_ulps)
    }
}
