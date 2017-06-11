//! This crate provides extension methods to `proc-macro`, and `proc-macro2`'s
//! `Literal` types. These methods provide a mechanism for extracting the value
//! of the type.
//!
//! ## Supported Features
//!
//! * `i128`: Add support for interpreting the `i128` and `u128` integer types.
//!   *nightly only*
//!
//! * `proc-macro2` **default**: Implement `LiteralExt` on `proc_macro2::Literal`.
//!
//! * `proc-macro`: Implement `LiteralExt` on `proc_macro::Literal`.
//!   *nightly only*
//!
//! * `dummy`: Export a type `DummyLiteral` with a public constructor
//!   which implements the `LiteralExt` trait.

#![cfg_attr(feature = "i128", feature(i128_type))]
#![cfg_attr(feature = "proc-macro", feature(proc_macro))]

#[cfg(feature = "proc-macro")]
extern crate proc_macro;

#[cfg(feature = "proc-macro2")]
extern crate proc_macro2;

#[cfg(feature = "dummy")]
use std::fmt;

mod internal;
mod test;

/// A dummy literal type to be used for testing or parsing literals, without
/// depending on either `proc-macro` or `proc-macro2`. Parses the result of
/// invoking `to_string` on its parameter.
///
/// ## Warning
///
/// This type is easy to misuse. When given an invalid input, parsers in this
/// crate may do the wrong thing or panic. This crate does not validate its
/// inputs.
#[cfg(feature = "dummy")]
pub struct DummyLiteral<T: fmt::Display>(pub T);
#[cfg(feature = "dummy")]
impl<T: fmt::Display> fmt::Display for DummyLiteral<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(not(feature = "i128"))]
type RawInt = u64;
#[cfg(feature = "i128")]
type RawInt = u128;

/// A type which represents an integer literal.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct IntLit {
    val: Option<RawInt>, // NOTE: Could be `None` if the value overflows.
    suffix: &'static str,
}

macro_rules! as_int_type {
    ($name:ident, $t:ident) => {
        /// Returns `None` if the value overflows, or if the suffix is wrong.
        pub fn $name(&self) -> Option<$t> {
            if self.suffix != "" &&
                self.suffix != stringify!($t) {
                return None;
            }
            self.val.and_then(|v| {
                if v > ($t::max_value() as RawInt) {
                    None
                } else {
                    Some(v as $t)
                }
            })
        }
    }
}

impl IntLit {
    /// Get the suffix written on the integer literal.
    pub fn suffix(&self) -> &str {
        &self.suffix
    }

    as_int_type!(as_u8, u8);
    as_int_type!(as_i8, i8);
    as_int_type!(as_u16, u16);
    as_int_type!(as_i16, i16);
    as_int_type!(as_u32, u32);
    as_int_type!(as_i32, i32);
    as_int_type!(as_u64, u64);
    as_int_type!(as_i64, i64);
    #[cfg(feature = "i128")]
    as_int_type!(as_u128, u128);
    #[cfg(feature = "i128")]
    as_int_type!(as_i128, i128);
}

/// A type which represents a floating point value.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FloatLit {
    val: f64,
    suffix: &'static str,
}

macro_rules! as_float_type {
    ($name:ident, $t:ident) => {
        /// Returns `None` if the suffix does not match the requested type.
        pub fn $name(&self) -> Option<$t> {
            if self.suffix != "" && self.suffix != stringify!($t) {
                return None
            } else {
                Some(self.val as $t)
            }
        }
    }
}

impl FloatLit {
    /// Get the suffix for the float.
    pub fn suffix(&self) -> &str {
        &self.suffix
    }

    as_float_type!(as_f32, f32);
    as_float_type!(as_f64, f64);
}

pub trait LiteralExt {
    /// If the `Literal` is an integer literal, returns its value.
    fn as_int(&self) -> Option<IntLit>;

    /// If the `Literal` is a floating point literal, returns its value.
    fn as_float(&self) -> Option<FloatLit>;

    /// If the `Literal` is a string literal, returns it's value.
    fn as_string(&self) -> Option<String>;

    /// If the `Literal` is a char literal, returns it's value.
    fn as_char(&self) -> Option<char>;

    /// If the `Literal` is a byte string literal, returns it's value.
    fn as_bytes(&self) -> Option<Vec<u8>>;

    /// If the `Literal` is a byte literal, returns it's value.
    fn as_byte(&self) -> Option<u8>;

    /// If the `Literal` is an inner doc comment (`//!` or `/*!`), returns a
    /// string with the text of the comment.
    fn as_inner_doc(&self) -> Option<String>;

    /// If the `Literal` is an outer doc comment (`///` or `/**`), returns a
    /// string with the text of the comment.
    fn as_outer_doc(&self) -> Option<String>;
}

macro_rules! impl_literal {
    () => {
        fn as_int(&self) -> Option<IntLit> {
            $crate::internal::int_lit(&self.to_string())
        }

        fn as_float(&self) -> Option<FloatLit> {
            $crate::internal::float_lit(self.to_string())
        }

        fn as_string(&self) -> Option<String> {
            $crate::internal::str_lit(&self.to_string())
        }

        fn as_char(&self) -> Option<char> {
            $crate::internal::char_lit(&self.to_string())
        }

        fn as_bytes(&self) -> Option<Vec<u8>> {
            $crate::internal::byte_str_lit(&self.to_string())
        }

        fn as_byte(&self) -> Option<u8> {
            $crate::internal::byte_lit(&self.to_string())
        }

        fn as_inner_doc(&self) -> Option<String> {
            $crate::internal::inner_doc(self.to_string())
        }

        fn as_outer_doc(&self) -> Option<String> {
            $crate::internal::outer_doc(self.to_string())
        }
    }
}

#[cfg(feature = "dummy")]
impl<T: fmt::Display> LiteralExt for DummyLiteral<T> {
    impl_literal!();
}

#[cfg(feature = "proc-macro")]
impl LiteralExt for proc_macro::Literal {
    impl_literal!();
}

#[cfg(feature = "proc-macro2")]
impl LiteralExt for proc_macro2::Literal {
    impl_literal!();
}
