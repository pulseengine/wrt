use wrt_error::Result;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

/// Trait for types that can be validated
pub trait Validatable {
    /// Validate that this object is well-formed
    fn validate(&self) -> Result<()>;
}

/// Simple validation helper for Option<T> where T is Validatable
impl<T: Validatable> Validatable for Option<T> {
    fn validate(&self) -> Result<()> {
        match self {
            Some(value) => value.validate(),
            None => Ok(()),
        }
    }
}

/// Simple validation helper for Vec<T> where T is Validatable
impl<T: Validatable> Validatable for Vec<T> {
    fn validate(&self) -> Result<()> {
        for item in self {
            item.validate()?;
        }
        Ok(())
    }
}
