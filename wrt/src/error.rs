// Re-export all public items from wrt-error
pub use wrt_error::*;

// Add From implementation for wrt_instructions::Error
impl From<wrt_instructions::Error> for Error {
    fn from(err: wrt_instructions::Error) -> Self {
        // wrt_instructions::Error already wraps wrt_error::Error
        // We can just use the inner error
        err.to_inner_error()
    }
}
