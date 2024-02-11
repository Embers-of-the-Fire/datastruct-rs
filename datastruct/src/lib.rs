//! # DataStruct.rs
//!
//! The library provides a derive macro to automatically implement "plain methods" for data structures.
//!
//! Currently Available:
//! - Default: Standard `Default`, lib-specific `DataStruct::data_default` and constant default `ConstDataStruct::DEFAULT`.
//! - Debug: Manual `Debug` filter.
//! - Comparison: Standard `Eq`, `PartialEq`, `Ord`, `PartialOrd`.
//! - Operations: Standard `Add(Assign)`, `Sub(Assign)`, `Mul(Assign)`, `Div(Assign)`.
//!
//! Unlike standard derive macros, the `DataStruct` macro accepts user-defined behaviors without
//! writing implementation code.

mod traits;
pub use traits::{DataStruct, ConstDataStruct};
pub use datastruct_derive::DataStruct;
