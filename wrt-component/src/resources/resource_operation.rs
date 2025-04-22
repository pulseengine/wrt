/// Operations that can be performed on resources
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceOperation {
    /// Read access to a resource
    Read,
    /// Write access to a resource
    Write,
    /// Execute a resource as code
    Execute,
    /// Create a new resource
    Create,
    /// Delete an existing resource
    Delete,
}

impl ResourceOperation {
    /// Check if the operation requires read access
    pub fn requires_read(&self) -> bool {
        match self {
            ResourceOperation::Read | ResourceOperation::Execute => true,
            _ => false,
        }
    }
    
    /// Check if the operation requires write access
    pub fn requires_write(&self) -> bool {
        match self {
            ResourceOperation::Write | ResourceOperation::Create | ResourceOperation::Delete => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
    }
} 