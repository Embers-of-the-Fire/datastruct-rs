pub trait SynErrorExt {
    fn update_or_combine(&mut self, err: syn::Error);
}

impl SynErrorExt for Option<syn::Error> {
    fn update_or_combine(&mut self, err: syn::Error) {
        match self {
            Some(e) => e.combine(err),
            None => *self = Some(err),
        }
    }
}

pub trait ResultExt {
    type OkValue;
    type ErrValue;

    fn swap(self) -> Result<Self::OkValue, Self::ErrValue>;
}

impl<T, E> ResultExt for Result<T, E> {
    type OkValue = E;
    type ErrValue = T;

    fn swap(self) -> Result<E, T> {
        match self {
            Ok(v) => Err(v),
            Err(v) => Ok(v),
        }
    }
}
