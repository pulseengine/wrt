// Re-export ResourceOperation for convenience
pub use wrt_foundation::resource::ResourceOperation;

use crate::prelude::*;

/// Convert from local ResourceOperation enum to format ResourceOperation
pub fn to_format_resource_operation(
    op: ResourceOperation,
    type_idx: u32,
) -> wrt_format::component::FormatResourceOperation {
    use wrt_format::component::FormatResourceOperation as FormatOp;
    use wrt_foundation::resource::{
        ResourceDrop,
        ResourceNew,
        ResourceRep,
    };

    match op {
        ResourceOperation::Read => FormatOp::Rep(ResourceRep { type_idx }),
        ResourceOperation::Write => FormatOp::Rep(ResourceRep { type_idx }), // Map to Rep
        ResourceOperation::Execute => FormatOp::Rep(ResourceRep { type_idx }), // Map to Rep
        ResourceOperation::Create => FormatOp::New(ResourceNew { type_idx }),
        ResourceOperation::Delete => FormatOp::Drop(ResourceDrop { type_idx }),
        ResourceOperation::Reference => FormatOp::Rep(ResourceRep { type_idx }), // Map to Rep
        ResourceOperation::Dereference => FormatOp::Rep(ResourceRep { type_idx }), // Map to Rep
    }
}

/// Convert from format ResourceOperation to local ResourceOperation
pub fn from_format_resource_operation(
    op: &wrt_format::component::FormatResourceOperation,
) -> ResourceOperation {
    use wrt_format::component::FormatResourceOperation as FormatOp;

    match op {
        FormatOp::Rep(_) => ResourceOperation::Read,
        FormatOp::New(_) => ResourceOperation::Create,
        FormatOp::Drop(_) => ResourceOperation::Delete,
    }
}

/// Convert a Core ResourceOperation to a Format ResourceOperation
#[cfg(not(feature = "safe-memory"))]
pub fn core_to_format_resource_operation(
    op: &wrt_foundation::ResourceOperation,
) -> ResourceOperation {
    match op {
        wrt_foundation::ResourceOperation::New => ResourceOperation::New,
        wrt_foundation::ResourceOperation::Drop => ResourceOperation::Drop,
        wrt_foundation::ResourceOperation::Rep => ResourceOperation::Rep,
    }
}

/// Convert a Format ResourceOperation to a Core ResourceOperation
#[cfg(not(feature = "safe-memory"))]
pub fn format_to_core_resource_operation(
    op: &ResourceOperation,
) -> wrt_foundation::ResourceOperation {
    match op {
        ResourceOperation::New => wrt_foundation::ResourceOperation::New,
        ResourceOperation::Drop => wrt_foundation::ResourceOperation::Drop,
        ResourceOperation::Rep => wrt_foundation::ResourceOperation::Rep,
    }
}

#[cfg(feature = "safe-memory")]
mod safe_memory {
    use wrt_foundation::ResourceOperation as FormatOp;

    use crate::prelude::*;

    /// Convert a Core ResourceOperation to a Format ResourceOperation
    pub fn core_to_format_resource_operation(op: &wrt_foundation::ResourceOperation) -> FormatOp {
        match op {
            wrt_foundation::ResourceOperation::New => FormatOp::New,
            wrt_foundation::ResourceOperation::Drop => FormatOp::Drop,
            wrt_foundation::ResourceOperation::Rep => FormatOp::Rep,
        }
    }

    /// Convert a Format ResourceOperation to a Core ResourceOperation
    pub fn format_to_core_resource_operation(op: &FormatOp) -> wrt_foundation::ResourceOperation {
        match op {
            FormatOp::New => wrt_foundation::ResourceOperation::New,
            FormatOp::Drop => wrt_foundation::ResourceOperation::Drop,
            FormatOp::Rep => wrt_foundation::ResourceOperation::Rep,
        }

}
