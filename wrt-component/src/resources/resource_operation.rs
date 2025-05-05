use crate::prelude::*;
use wrt_types::resource::ResourceOperation;

// Re-export ResourceOperation for convenience
pub use wrt_types::resource::ResourceOperation;

/// Convert from local ResourceOperation enum to format ResourceOperation
pub fn to_format_resource_operation(
    op: ResourceOperation,
    type_idx: u32,
) -> wrt_format::component::ResourceOperation {
    use wrt_format::component::ResourceOperation as FormatOp;
    use wrt_types::resource::{ResourceDrop, ResourceNew, ResourceRep};

    match op {
        ResourceOperation::Read => FormatOp::Rep(ResourceRep { type_idx }),
        ResourceOperation::Write => FormatOp::Transfer,
        ResourceOperation::Execute => FormatOp::Execute,
        ResourceOperation::Create => FormatOp::New(ResourceNew { type_idx }),
        ResourceOperation::Delete => FormatOp::Drop(ResourceDrop { type_idx }),
        ResourceOperation::Reference => FormatOp::Borrow,
        ResourceOperation::Dereference => FormatOp::Dereference,
    }
}

/// Convert from format ResourceOperation to local ResourceOperation
pub fn from_format_resource_operation(
    op: &wrt_format::component::ResourceOperation,
) -> ResourceOperation {
    use wrt_format::component::ResourceOperation as FormatOp;

    match op {
        FormatOp::Rep(_) => ResourceOperation::Read,
        FormatOp::Transfer => ResourceOperation::Write,
        FormatOp::Execute => ResourceOperation::Execute,
        FormatOp::New(_) => ResourceOperation::Create,
        FormatOp::Drop(_) => ResourceOperation::Delete,
        FormatOp::Destroy(_) => ResourceOperation::Delete,
        FormatOp::Borrow => ResourceOperation::Reference,
        FormatOp::Dereference => ResourceOperation::Dereference,
        _ => ResourceOperation::Read, // Default to read for unknown operations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_format::component::ResourceOperation as FormatOp;
    use wrt_types::resource::{ResourceDrop, ResourceNew, ResourceRep};

    #[test]
    fn test_operation_permissions() {
        assert!(ResourceOperation::Read.requires_read());
        assert!(!ResourceOperation::Read.requires_write());

        assert!(ResourceOperation::Write.requires_write());
        assert!(!ResourceOperation::Write.requires_read());

        assert!(ResourceOperation::Execute.requires_read());
        assert!(!ResourceOperation::Execute.requires_write());

        assert!(ResourceOperation::Create.requires_write());
        assert!(!ResourceOperation::Create.requires_read());

        assert!(ResourceOperation::Delete.requires_write());
        assert!(!ResourceOperation::Delete.requires_read());

        assert!(ResourceOperation::Reference.requires_write());
        assert!(!ResourceOperation::Reference.requires_read());

        assert!(ResourceOperation::Dereference.requires_read());
        assert!(!ResourceOperation::Dereference.requires_write());
    }

    #[test]
    fn test_format_conversion() {
        // Test conversion to format types
        let type_idx = 42;

        let read_op = to_format_resource_operation(ResourceOperation::Read, type_idx);
        if let FormatOp::Rep(rep) = read_op {
            assert_eq!(rep.type_idx, type_idx);
        } else {
            panic!("Unexpected operation type");
        }

        let create_op = to_format_resource_operation(ResourceOperation::Create, type_idx);
        if let FormatOp::New(new) = create_op {
            assert_eq!(new.type_idx, type_idx);
        } else {
            panic!("Unexpected operation type");
        }

        // Test conversion from format types
        assert_eq!(
            from_format_resource_operation(&FormatOp::Rep(ResourceRep { type_idx })),
            ResourceOperation::Read
        );

        assert_eq!(
            from_format_resource_operation(&FormatOp::New(ResourceNew { type_idx })),
            ResourceOperation::Create
        );

        assert_eq!(
            from_format_resource_operation(&FormatOp::Drop(ResourceDrop { type_idx })),
            ResourceOperation::Delete
        );
    }
}
