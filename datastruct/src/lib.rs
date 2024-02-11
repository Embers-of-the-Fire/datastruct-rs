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
//!
//! ## Basic Syntax
//!
//! To configure the meta options of the generator, add `#[dstruct(options)]` before your definition.
//!
//! To configure the fields' options, add `#[dfield(options)]` before your field declaration.
//!
//! ```rust
//! use datastruct_derive::DataStruct;
//!
//! #[derive(DataStruct)]
//! #[dstruct()]
//! struct Data {
//!     #[dfield()]
//!     field: u32,
//! }
//! ```
//!
//! You can define the meta attributes multiple times,
//! and the last declaration of each attribute will be used to generate the code.
//!
//! For example,
//!
//! ```rust
//! # use datastruct_derive::DataStruct;
//!
//! # #[derive(DataStruct)]
//! # #[dstruct()]
//! # struct Data {
//! #[dfield(no_debug, cmp(eq = false))]
//! #[dfield(cmp(eq = true))]
//! #     field: u32,
//! # }
//! ```
//!
//! is equivalent to
//!
//! ```rust
//! # use datastruct_derive::DataStruct;
//!
//! # #[derive(DataStruct)]
//! # #[dstruct()]
//! # struct Data {
//! #[dfield(no_debug, cmp(eq = true))]
//! #     field: u32,
//! # }
//! ```
//!
//! ## Api Document
//!
//! ### Default
//!
//! #### Default Implementation
//!
//! All default implementations are like:
//!
//! ```rust,ignore
//! struct NeedDefault {
//!     field1: usize,
//!     field2: String,
//! }
//!
//! // generated code
//! let default: NeedDefault = {
//!     let field1: usize = 42;
//!     let field2: String = "Something default".to_string();
//!     NeedDefault {
//!         field1,
//!         field2,
//!     }
//! };
//! ```
//!
//! That means you can refer to other fields when initializing the default value.
//!
//! ```rust,ignore
//! use datastruct::DataStruct;
//!
//! #[derive(DataStruct)]
//! #[dstruct(default)]
//! struct MyDefault {
//!     #[dfield(default = "second + first")]
//!     manual: usize,
//!     #[dfield(default = "40", seq = -1)]
//!     second: usize,
//!     #[dfield(default = "2", seq = -2)]
//!     first: usize,
//! }
//!
//! // generated code
//! impl ::datastruct::DataStruct for MyDefault {
//!     fn data_default() -> Self {
//!         let first: usize = 2;
//!         let second: usize = 40;
//!         let manual: usize = second + first;
//!         Self {
//!             first,
//!             second,
//!             manual,
//!         }
//!     }
//! }
//! ```
//!
//! #### `default`
//!
//! Ask the macro to generate an implementation of `datastruct::DataStruct`,
//! which offers a runtime-default value but does not pollute the namespace.
//!
//! **Syntax:**
//! - `#[dstruct(default)]`
//!
//! **Restriction:**
//! - All fields must be provided with default value.
//!
//! **Field Configuration:**
//! - `#[dfield(default = xxx)]`
//!
//!   The literal expression should be wrapped inside a string, like this:
//!
//!   ```rust
//!   # use datastruct_derive::DataStruct;
//!
//!   # #[derive(DataStruct)]
//!   # #[dstruct()]
//!   # struct Data {
//!   #[dfield(default = "field + another")]
//!   #     dfield: usize,
//!   #[dfield(default = "42_usize")]
//!   #     field: usize,
//!   #     #[dfield(default = "0_usize")]
//!   #     another: usize,
//!   # }
//!   #
//!   ```
//!
//!   If no default value is provided, the field will be considered uninitialized
//!   and `default`-related code cannot be generated.
//! - `#[dfield(seq = xxx)]` | `#[dfield(sequence = xxx)]` where `xxx` is `isize`
//!
//!   Change the sequence of the fields. By default, the sequence to initialize the fields
//!   is the same as how they are declared. The sequence option can help reorganize the process.
//!   Fields that are not tagged with `seq = isize` will inherit from the default sequence.
//!   You can also set the index with negative numbers.
//!
//!   ```rust
//!   # use datastruct_derive::DataStruct;
//!
//!   # #[derive(DataStruct)]
//!   #[dstruct(default)]
//!   struct Data {
//!       field: u8,  // seq = 0
//!       #[dfield(seq = -1)]
//!       before: u8, // default seq = 1, seq = -1
//!   }
//!   ```
//!
//! #### `const`
//!
//! Ask the macro to generate an implementation of `datastruct::ConstDataStruct`,
//! which offers a compile-time const default value but does not pollute the namespace.
//!
//! **Syntax:**
//! - `#[dstruct(const)]`
//!
//! **Restriction:**
//! - All fields must be provided with **const** default value.
//!
//! **Field Configuration:**
//! - Inherits from `default`.
//!
//! #### `std_default`
//!
//! The same as `default`, but implement `std::default::Default` instead.
//!
//! **Syntax:**
//! - `#[dstruct(std_default)]`
//!
//! **Restriction:**
//! - All fields must be provided with default value.
//!
//! **Field Configuration:**
//! - Inherits from `default`.
//!
//! **Warning:**
//! - This may pollute the namespace, and IDE may not be able to identify the implementation.
//!
//! #### `partial`
//!
//! Partially default implementation.
//!
//! This will produce something like:
//!
//! ```rust,ignore
//! use datastruct::DataStruct;
//!
//! #[derive(DataStruct)]
//! #[dstruct(partial)]
//! struct Data {
//!     #[dfield(default = "10")]
//!     value1: u32,
//!     value2: u32,
//! }
//!
//! // generated code
//! impl Data {
//!     pub fn partial_default(value2: u32) -> Self {
//!         Self {
//!             value1: 10,
//!             value2,
//!         }
//!     }
//! }
//! ```
//!
//! **Syntax:**
//! - `#[dstruct(partial)]`
//!
//! **Field Configuration:**
//! - Inherits from `default`.
//!
//! ### Setter and Getter
//!
//! #### `set`
//!
//! Generate default setter for fields.
//!
//! **Setter Type:**
//! - `full` | `all`: Both `set` and `with`. (Default.)
//! - `set`: `set_field_name(&mut self, value)`. Set the structure's value.
//! - `with`: `with_field_name(self, value) -> Self`. Internally set the structure's value and return itself.
//! - `no`: Ignore the field.
//!
//! **Syntax:**
//! - `#[dstruct(set)]`: Default setter configuration.
//! - `#[dstruct(set = "setter_type")]`: Set default setter configuration to `setter_type`.
//!
//! **Field Configuration:**
//! - `#[dfield(set)]`: Inherit the setter configuration from the structure. Typically, you don't need to specify this.
//! - `#[dfield(set = "setter_type")]`: Override the default setter configuration.
//!
//! **Example:**
//!
//! ```rust,ignore
//! use datastruct::DataStruct;
//!
//! #[derive(DataStruct)]
//! #[dstruct(set)] // equivalent to `set = "full"`
//! struct Data {
//!     auto_set: usize,
//!     #[dfield(set = "no")]
//!     no_set: usize,
//!     #[dfield(set = "set")]
//!     only_set: usize,
//! }
//!
//! // generated code
//! impl Data {
//!     pub fn set_auto_set(&mut self, auto_set: usize) {
//!         self.auto_set = auto_set;
//!     }
//!     pub fn set_only_set(&mut self, only_set: usize) {
//!         self.only_set = only_set;
//!     }
//!
//!     pub fn with_auto_set(mut self, auto_set: usize) -> Self {
//!         self.auto_set = auto_set;
//!         self
//!     }
//! }
//! ```
//!
//!
//! #### `get`
//!
//! Generate default getter for fields.
//!
//! **Setter Type:**
//! - `full` | `all`: Both `move` and `get`.
//! - `get`: `field_name(&self) -> &value`. Get the structure's field's reference. (Default.)
//! - `move`: `get_field_name(self) -> move`. Move the field out of the structure.
//! - `no`: Ignore the field.
//!
//! **Syntax:**
//! - `#[dstruct(get)]`: Default getter configuration.
//! - `#[dstruct(get = "getter_type")]`: Set default getter configuration to `getter_type`.
//!
//! **Field Configuration:**
//! - `#[dfield(get)]`: Inherit the getter configuration from the structure. Typically, you don't need to specify this.
//! - `#[dfield(get = "getter_type")]`: Override the default getter configuration.
//!
//! **Example:**
//!
//! ```rust,ignore
//! use datastruct::DataStruct;
//!
//! #[derive(DataStruct)]
//! #[dstruct(get)] // equivalent to `get = "get"`
//! struct Data {
//!     auto_get: usize,
//!     #[dfield(get = "no")]
//!     no_get: usize,
//!     #[dfield(get = "full")]
//!     can_move: usize,
//! }
//!
//! // generated code
//! impl Data {
//!     pub fn auto_get(&self) -> &usize {
//!         &self.auto_get
//!     }
//!     pub fn can_move(&self) -> &usize {
//!         &self.can_move
//!     }
//!
//!     pub fn get_can_move(self) -> usize {
//!         self.can_move
//!     }
//! }
//! ```
//!
//! #### `map`
//!
//! Map a field's value and modify the structure. This does not have structure-level configuration.
//!
//! **Field Configuration:**
//! - `#[dfield(map)]` | `#[dfield(map = ture)`: Enable mapping.
//! - `#[dfield(map = false)]`: Disable mapping.
//!   Typically, you don't need to explicitly disable mapping since it's the default behavior.
//!
//! **Examples:**
//!
//! ```rust,ignore
//! use datastruct::DataStruct;
//!
//! #[derive(DataStruct)]
//! struct MapItem {
//!     #[dfield(map)] // equivalent to `#[dfield(map = true)]
//!     item: usize,
//! }
//!
//! // generated code
//! impl MapItem {
//!     pub fn map_item(mut self, f: impl FnOnce(usize) -> usize) -> Self {
//!         self.item = f(self.item);
//!         self
//!     }
//! }
//! ```
//!
//! #### `do_with`
//!
//! Modify a field's value. This does not have structure-level configuration.
//!
//! **Field Configuration:**
//! - `#[dfield(do_with)]` | `#[dfield(do_with = ture)`: Enable `do_with`.
//! - `#[dfield(do_with = false)]`: Disable `do_with`.
//!   Typically, you don't need to explicitly disable mapping since it's the default behavior.
//!
//! **Examples:**
//!
//! ```rust,ignore
//! use datastruct::DataStruct;
//!
//! #[derive(DataStruct)]
//! struct MapItem {
//!     #[dfield(do_with)] // equivalent to `#[dfield(do_with = true)]
//!     item: usize,
//! }
//!
//! // generated code
//! impl MapItem {
//!     pub fn do_with_item(&mut self, f: impl FnOnce(&mut usize)) {
//!         f(&mut self.item);
//!     }
//! }
//! ```
//!
//! ### Comparison `cmp`
//!
//! Macro-generateable comparison traits are `Eq`, `PartialEq`, `Ord` and `PartialOrd`.
//!
//! **Syntax:**
//!
//! All `cmp` configurations must be defined within `cmp(xxx)` field:
//!
//! ```rust,ignore
//! #[dstruct(cmp(<your config>))]
//! #[dfield(cmp(<your config>))]
//! ```
//!
//! #### `Eq` and `PartialEq`
//!
//! **Syntax:**
//! - `#[dstruct(cmp(eq))]`: Generate `Eq` implementation for the struct.
//!   Note that this won't implement `PartialEq`, and you must explicitly enable that.
//! - `#[dfield(cmp(peq))]` | `#[dfield(cmp(partial_eq))]`: Generate `PartialEq` implementation for the struct.
//!
//! **Field Configuration:**
//! - `#[dfield(cmp(eq))]`: When checking equality, this field is included. (Default if enabled.)
//! - `#[dfield(cmp(eq = boolean))]`: Whether to include this field in equality check.
//!
//! **Examples:**
//!
//! ```rust,ignore
//! use datastruct::DataStruct;
//!
//! #[derive(DataStruct)]
//! #[dstruct(cmp(eq, peq))]
//! struct CanEq {
//!     // #[dfield(cmp(eq))]
//!     // you don't need to explicitly specify this.
//!     content: usize,
//!     #[dfield(cmp(eq = false))]
//!     do_not_check: usize,
//! }
//!
//! // generated code
//! impl ::std::cmp::PartialEq for CanEq {
//!     fn eq(&self, rhs: &Self) -> bool {
//!       (self.content == rhs.content)
//!     }
//! }
//! impl ::std::cmp::Eq for CanEq {}
//! ```
//!
//! #### `Ord` and `PartialOrd`
//!
//! **Syntax:**
//! - `#[dstruct(ord)]`: Implement `Ord` for the struct.
//! - `#[dstruct(pord)]` | `#[dstruct(partial_ord)]`: Implement `PartialOrd` for the struct.
//!
//! **Field Configuration:**
//! - `Ord`: The configuration key is `cmp` or `ord`. (Disabled by default.)
//!   - `#[dfield(cmp(ord))]`: Include this field in the `Ord` implementation.
//!   - `#[dfield(cmp(ord = boolean))]`: Whether to include this field in the `Ord` implementation.
//!   - `#[dfield(cmp(ord = "isize"))]` | `#[dfield(cmp(ord = isize))]`:
//!     Set the sequence of the field in the `Ord` implementation.
//!
//!     By default, all included fields' comparison results are chained with
//!     [`Ordering::then_with`](https://doc.rust-lang.org/std/cmp/enum.Ordering.html#method.then_with).
//!     This configuration can change the index of the field. Negative number is allowed to use.
//! - `PartialOrd`: The configuration key is `pcmp`, `partial_cmp`, `pord` or `partial_ord`. (Disabled by default.)
//!   - `#[dfield(cmp(pord))]`: Include this field in the `PartialOrd` implementation.
//!   - `#[dfield(cmp(pord = boolean))]`: Whether to include this field in the `PartialOrd` implementation.
//!   - `#[dfield(cmp(pord = "isize"))]` | `#[dfield(cmp(pord = isize))]`:
//!     Set the sequence of the field in the `PartialOrd` implementation.
//!
//!     By default, all included fields' comparison results are chained with
//!     [`Option::and_then`](https://doc.rust-lang.org/std/option/enum.Option.html#method.and_then) and
//!     [`Ordering::then_with`](https://doc.rust-lang.org/std/cmp/enum.Ordering.html#method.then_with).
//!     This configuration can change the index of the field. Negative number is allowed to use.
//!
//! **Note:**
//! - If no field is configured to be included, then `Ord` and `PartialOrd` will not be implemented.
//! - If both `Ord` and `PartialOrd` are enabled:
//!   - If only `Ord` is configured, then `PartialOrd` will be simply `Some(Ord)`.
//!   - If both are configured, Clippy may throw a `clippy::non_canonical_partial_ord_impl`
//!     (non-canonical implementation of `partial_cmp` on an `Ord` type) warning about the implementation, see
//!     [Clippy Lint](https://rust-lang.github.io/rust-clippy/master/index.html#non_canonical_partial_ord_impl)
//!     for more information.
//!
//! **Examples:**
//!
//! ```rust,ignore
//! use datastruct::DataStruct;
//!
//! #[derive(DataStruct)]
//! #[dstruct(cmp(eq, peq, ord, pord))]
//! struct MyComparable {
//!     #[dfield(cmp(ord))]
//!     only_ord: usize,
//!     #[dfield(cmp(pord))]
//!     only_partial_ord: usize,
//!     #[dfield(cmp(ord = -1, pord = -1))]
//!     both_ord: usize,
//! }
//!
//! // generated code (`Eq` and `PartialEq` is omitted).
//!
//! impl ::std::cmp::Ord for MyComparable {
//!     fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
//!         self.both_ord
//!             .cmp(&other.both_ord)
//!             .then_with(|| self.only_ord.cmp(&other.only_ord))
//!     }
//! }
//!
//! impl ::std::cmp::PartialOrd for MyComparable {
//!     fn partial_cmp(&self, other: &Self) -> ::core::option::Option<::std::cmp::Ordering> {
//!         self.both_ord
//!             .partial_cmp(&other.both_ord)
//!             // the following identifier `__gen_xxx` is generated by the macro
//!             .and_then(|__gen_dparord| {
//!                 self.only_partial_ord
//!                     .partial_cmp(&other.only_partial_ord)
//!                     .map(|__gen_dparord_self| __gen_dparord.then(__gen_dparord_self))
//!             })
//!     }
//! }
//! ```
//!
//! ### Operations `ops`
//!
//! Macro-generateable operation traits are `Add +`, `Sub -`, `Mul *`, `Div /`
//! and their assignable versions `AddAssign +=`, `SubAssign -=`, `MulAssign *=` and `DivAssign /=`.
//!
//! **Syntax:**
//!
//! All `ops` configurations must be defined within `ops(xxx)` field:
//!
//! ```rust,ignore
//! #[dstruct(ops(<your config>))]
//! #[dfield(ops(<your config>))]
//! ```
//!
//! Operations definitions on structure-level can be declared by
//! `#[dstruct(ops(add = "type"))]` (Take `Add/AddAssign` as an example):
//! - "both" | "all": Generate both assignment and plain operation for all fields by default.
//! - "assign": Only generate assignment by default.
//! - "plain" | "default"": Generate plain operation by default. (Default if enabled.)
//!
//! **Field Configuration:**
//! - Plain operations `+ - * /`: (Take `Add +` as an example:)
//!   - `#[dfield(ops(add = "type"))]`:
//!     - "inherit" | "default": Inherit the default configuration declared in the `dstruct` attributes.
//!     - "ignore" | "no": Ignore this field, that is, `A1 + A2 -> A1`.
//!   - `#[dfield(ops(add = boolean))]`: Whether to include this field.
//!   - `#[dfield(ops(add = "expression"))]`: Use your own expression to implement the `Add`.
//! - Assignment operations `+= -= *= /=`: (Take `AddAssign +=` as an example:)
//!   - `#[dfield(ops(add_assign = "type"))]`:
//!     - "inherit" | "default": Inherit the default configuration declared in the `dstruct` attributes.
//!     - "ignore" | "no": Ignore this field, that is, `self.A <- self.A`.
//!   - `#[dfield(ops(add_assign = boolean))]`: Whether to include this field.
//!   - `#[dfield(ops(add_assign = "expression"))]`: Use your own expression to implement the `AddAssign`.
//!
//! **About `expression`:**
//!
//! You can use your expression to manually implement the operations.
//! The expression must be wrapped in a literal string.
//! Use `$self` to refer to the left-hand `self` value, and use `$rhs` to refer to the right-hand `other` value.
//!
//! For example,
//!
//! ```rust,ignore
//! #[dfield(ops(add = "$self.something + $rhs.otherthing"))]
//! field: SomeType
//! ```
//!
//! will be translated into
//!
//! ```rust,ignore
//! Self {
//!     field: self.something + other.otherthing,
//!     ..
//! }
//! ```
//!
//! **Examples:**
//!
//! ```rust,ignore
//! use datastruct::DataStruct;
//!
//! #[derive(Debug, Clone, Copy, DataStruct)]
//! #[dstruct(ops(add = "both"))]
//! struct CanOpsAssign {
//!     can_add: i8,
//!     #[dfield(ops(add = "ignore"))]
//!     no_add: i8,
//!     #[dfield(ops(add_assign = "std::cmp::max($self.max, $rhs.max)"))]
//!     max: i8,
//!     #[dfield(ops(add_assign = "std::cmp::min($self.min, $rhs.min)"))]
//!     min: i8,
//! }
//!
//! // generated code
//! impl ::std::ops::Add for CanOpsAssign {
//!     type Output = Self;
//!     fn add(self, rhs: Self) -> Self {
//!         Self {
//!             can_add: self.can_add + rhs.can_add,
//!             no_add: self.no_add,
//!             max: self.max + rhs.max,
//!             min: self.min + rhs.min,
//!         }
//!     }
//! }
//!
//! impl ::std::ops::AddAssign for CanOpsAssign {
//!     fn add_assign(&mut self, rhs: Self) {
//!         self.can_add += rhs.can_add;
//!         self.no_add += rhs.no_add;
//!         self.max = std::cmp::max(self.max, rhs.max);
//!         self.min = std::cmp::min(self.min, rhs.min);
//!     }
//! }
//! ```


mod traits;
pub use traits::{DataStruct, ConstDataStruct};
pub use datastruct_derive::DataStruct;
