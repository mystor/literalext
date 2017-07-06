# `literalext`
[![Build Status](https://travis-ci.org/mystor/literalext.svg?branch=master)](https://travis-ci.org/mystor/literalext)

This crate provides extension methods to `proc-macro`, and `proc-macro2`'s
`Literal` types. These methods provide a mechanism for extracting the value of
the type.

## API

Adds a trait with implementations for the types `proc_macro2::Literal`,
`proc_macro::Literal`, and `DummyLiteral` with the following methods for
extracting the value of the type:

```rust
pub trait LiteralExt {
    /// If the `Literal` is an integer literal, returns its value.
    fn parse_int(&self) -> Option<IntLit>;

    /// If the `Literal` is a floating point literal, returns its value.
    fn parse_float(&self) -> Option<FloatLit>;

    /// If the `Literal` is a string literal, returns it's value.
    fn parse_string(&self) -> Option<String>;

    /// If the `Literal` is a char literal, returns it's value.
    fn parse_char(&self) -> Option<char>;

    /// If the `Literal` is a byte string literal, returns it's value.
    fn parse_bytes(&self) -> Option<Vec<u8>>;

    /// If the `Literal` is a byte literal, returns it's value.
    fn parse_byte(&self) -> Option<u8>;

    /// If the `Literal` is an inner doc comment (`//!` or `/*!`), returns a
    /// string with the text of the comment.
    fn parse_inner_doc(&self) -> Option<String>;

    /// If the `Literal` is an outer doc comment (`///` or `/**`), returns a
    /// string with the text of the comment.
    fn parse_outer_doc(&self) -> Option<String>;
}
```

## Supported Features

* `i128`: Add support for interpreting the `i128` and `u128` integer types.
  *nightly only*

* `proc-macro2` **default**: Implement `LiteralExt` on `proc_macro2::Literal`.

* `proc-macro`: Implement `LiteralExt` on `proc_macro::Literal`. *nightly only*

* `dummy`: Export a type `DummyLiteral` with a public constructor which
  implements the `LiteralExt` trait.
