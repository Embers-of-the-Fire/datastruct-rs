/// The macro's `default` option will implement this trait for the structure.
pub trait DataStruct {
    /// Returns the default value of the structure.
    fn data_default() -> Self;
}

/// The macro's `const` option will implement this trait for the structure.
pub trait ConstDataStruct {
    /// The constant default value of the structure.
    const DEFAULT: Self;
}
