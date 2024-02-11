# DataStruct.rs

This is a procedural macro library to automatically generate duplicate code for pure data structures.

## What can this lib do?

The library provides a derive macro to automatically implement "plain methods" for data structures.

Currently Available:
- Default: Standard `Default`, lib-specific `DataStruct::data_default` and constant default `ConstDataStruct::DEFAULT`.
- Debug: Manual `Debug` filter.
- Comparison: Standard `Eq`, `PartialEq`, `Ord`, `PartialOrd`.
- Operations: Standard `Add(Assign)`, `Sub(Assign)`, `Mul(Assign)`, `Div(Assign)`.

Unlike standard derive macros, the `DataStruct` macro accepts user-defined behaviors without
writing implementation code.

## Quick Start

> For full documentation, read it [here](./DOCUMENT.md).

Let's start with this example structure:

```rust
struct Person {
    age: u8,
    name: String,
    private_key: u32,
}
```

First, add `datastruct` to your dependencies. The core entry point of the library is `DataStruct` macro.

```rust
use datastruct::DataStruct;
#[derive(DataStruct)]
#[dstruct(debug)]
struct Person {
    age: u8,
    name: String,
    #[dfield(no_debug)]
    private_key: u32,
}
```

The `#[dstruct(xxx)]` is used to configure the basic options of the code-generator. In this example,
`debug` means that the `Debug` trait will be implemented.

The `#[dfield(xxx)]` is used to configure field-specific options of the code-generator. In this example,
`no_debug` means that this field will not be included in the debug output.

```rust
use datastruct::DataStruct;
#[derive(DataStruct)]
#[dstruct(debug)]
struct Person {
    age: u8,
    name: String,
    #[dfield(no_debug)]
    private_key: u32,
}

let person = Person { age: 22, name: "James".to_string(), private_key: 42 };
println!("{:#?}", person);
// Output:
// Person {
//     age: 22,
//     name: "James",
// }
```

## Limitations

Currently, the library can only generate code for typical structure, and tuple structure is not supported.

Besides, most IDE-support cannot offer full completion for macro-generated code, compared with manual implementation.
