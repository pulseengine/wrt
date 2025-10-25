//! Component Linker and Import/Export Resolution System

// Cross-environment imports
#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(feature = "std")]
use std::{
    boxed::Box,
    collections::HashMap,
    format,
    string::String,
    vec::Vec,
};

use wrt_error::{
    codes,
    Error,
    ErrorCategory,
    Result,
};
#[cfg(not(feature = "std"))]
use wrt_foundation::{
    collections::StaticVec as BoundedVec,
    bounded::BoundedString,
    budget_aware_provider::CrateId,
    safe_managed_alloc,
    safe_memory::NoStdProvider,
};

use crate::prelude::*;

// Type aliases for no_std environment with proper generics
#[cfg(not(feature = "std"))]
type String = BoundedString<256>;
#[cfg(not(feature = "std"))]
type Vec<T> = BoundedVec<T, 256>;
#[cfg(not(feature = "std"))]
type HashMap<K, V> = wrt_foundation::collections::StaticMap<K, V, 64>;

use crate::{
    components::{
        component::Component,
        component_instantiation::{
            create_component_export,
            create_component_import,
            ComponentExport,
            ComponentImport,
            ExportType,
            FunctionSignature,
            ImportType,
            InstanceConfig,
            InstanceId,
        },
    },
    types::ComponentInstance,
};

/// Maximum number of components in linker
const MAX_LINKED_COMPONENTS: usize = 256;

/// Component identifier in the linker
pub type ComponentId = String;

/// Component linker for managing multiple components and their dependencies
#[derive(Debug)]
pub struct ComponentLinker {
    /// Registered components
    components:       HashMap<ComponentId, ComponentDefinition>,
    /// Active component instances
    instances:        HashMap<InstanceId, ComponentInstance>,
    /// Dependency graph
    link_graph:       LinkGraph,
    /// Next available instance ID
    next_instance_id: InstanceId,
    /// Linker configuration
    config:           LinkerConfig,
    /// Resolution statistics
    stats:            LinkingStats,
}

/// Component definition in the linker
#[derive(Debug, Clone)]
pub struct ComponentDefinition {
    /// Component ID
    pub id:       ComponentId,
    /// Component binary (simplified as bytes)
    pub binary:   Vec<u8>, // Use Vec for std, BoundedVec handled in no_std type alias
    /// Parsed exports
    pub exports:  Vec<ComponentExport>,
    /// Parsed imports
    pub imports:  Vec<ComponentImport>,
    /// Component metadata
    pub metadata: ComponentMetadata,
}

/// Component metadata for introspection
#[derive(Debug, Clone)]
pub struct ComponentMetadata {
    /// Component name
    pub name:        String,
    /// Component version
    pub version:     String,
    /// Component description
    pub description: String,
    /// Component author
    pub author:      String,
    /// Compilation timestamp
    pub compiled_at: u64,
}

/// Dependency graph for component linking
#[derive(Debug, Clone)]
pub struct LinkGraph {
    /// Nodes (components)
    nodes: Vec<GraphNode>,
    /// Edges (dependencies)
    edges: Vec<GraphEdge>,
}

/// Graph node representing a component
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    /// Component ID
    pub component_id: ComponentId,
    /// Node index in graph
    pub index:        usize,
    /// Dependencies (outgoing edges)
    pub dependencies: Vec<usize>,
    /// Dependents (incoming edges)
    pub dependents:   Vec<usize>,
}

/// Graph edge representing a dependency relationship
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEdge {
    /// Source node index
    pub from:   usize,
    /// Target node index
    pub to:     usize,
    /// Import that creates this dependency
    pub import: ComponentImport,
    /// Export that satisfies this dependency
    pub export: ComponentExport,
    /// Edge weight (for optimization)
    pub weight: u32,
}

/// Linker configuration
#[derive(Debug, Clone)]
pub struct LinkerConfig {
    /// Enable strict type checking
    pub strict_typing:            bool,
    /// Allow hot swapping of components
    pub allow_hot_swap:           bool,
    /// Maximum memory per instance
    pub max_instance_memory:      u32,
    /// Enable dependency validation
    pub validate_dependencies:    bool,
    /// Circular dependency handling
    pub circular_dependency_mode: CircularDependencyMode,
}

/// Circular dependency handling modes
#[derive(Debug, Clone, PartialEq)]
pub enum CircularDependencyMode {
    /// Reject circular dependencies
    Reject,
    /// Allow circular dependencies (with limitations)
    Allow,
    /// Warn about circular dependencies but allow them
    Warn,
}

/// Linking statistics
#[derive(Debug, Clone, Default)]
pub struct LinkingStats {
    /// Total components registered
    pub components_registered: u32,
    /// Total instances created
    pub instances_created:     u32,
    /// Total links resolved
    pub links_resolved:        u32,
    /// Resolution failures
    pub resolution_failures:   u32,
    /// Last resolution time (microseconds)
    pub last_resolution_time:  u64,
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            strict_typing:            true,
            allow_hot_swap:           false,
            max_instance_memory:      64 * 1024 * 1024, // 64MB
            validate_dependencies:    true,
            circular_dependency_mode: CircularDependencyMode::Reject,
        }
    }
}

impl Default for ComponentMetadata {
    fn default() -> Self {
        #[cfg(feature = "std")]
        {
            Self {
                name:        String::new(),
                version:     "1.0.0".to_owned(),
                description: String::new(),
                author:      String::new(),
                compiled_at: 0,
            }
        }
        #[cfg(not(feature = "std"))]
        {
            use wrt_foundation::{budget_aware_provider::CrateId, safe_managed_alloc};

            let name_provider = safe_managed_alloc!(1024, CrateId::Component)
                .unwrap_or_else(|_| panic!("Failed to allocate memory for ComponentMetadata name"));
            let version_provider = safe_managed_alloc!(1024, CrateId::Component)
                .unwrap_or_else(|_| panic!("Failed to allocate memory for ComponentMetadata version"));
            let description_provider = safe_managed_alloc!(1024, CrateId::Component)
                .unwrap_or_else(|_| panic!("Failed to allocate memory for ComponentMetadata description"));
            let author_provider = safe_managed_alloc!(1024, CrateId::Component)
                .unwrap_or_else(|_| panic!("Failed to allocate memory for ComponentMetadata author"));

            Self {
                name:        BoundedString::try_from_str("")
                    .unwrap_or_else(|_| panic!("Failed to create ComponentMetadata name")),
                version:     BoundedString::try_from_str("1.0.0")
                    .unwrap_or_else(|_| panic!("Failed to create ComponentMetadata version")),
                description: BoundedString::try_from_str("")
                    .unwrap_or_else(|_| panic!("Failed to create ComponentMetadata description")),
                author:      BoundedString::try_from_str("")
                    .unwrap_or_else(|_| panic!("Failed to create ComponentMetadata author")),
                compiled_at: 0,
            }
        }
    }
}

impl ComponentLinker {
    /// Create a new component linker
    pub fn new() -> Self {
        Self::with_config(LinkerConfig::default())
    }

    /// Create a new component linker with custom configuration
    pub fn with_config(config: LinkerConfig) -> Self {
        Self {
            components: HashMap::new(),
            instances: HashMap::new(),
            link_graph: LinkGraph::new(),
            next_instance_id: 1,
            config,
            stats: LinkingStats::default(),
        }
    }

    /// Add a component to the linker
    pub fn add_component(&mut self, id: ComponentId, binary: &[u8]) -> Result<()> {
        if self.components.len() >= MAX_LINKED_COMPONENTS {
            return Err(Error::resource_exhausted(
                "Maximum number of components reached",
            ));
        }

        // Parse component binary (simplified)
        let (exports, imports, metadata) = self.parse_component_binary(binary)?;

        // Convert binary slice to Vec
        #[cfg(feature = "std")]
        let binary_vec = binary.to_vec();

        #[cfg(not(feature = "std"))]
        let binary_vec = {
            let mut vec = Vec::new();
            for &byte in binary {
                vec.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Binary too large for component",
                    )
                })?;
            }
            vec
        };

        let definition = ComponentDefinition {
            id: id.clone(),
            binary: binary_vec,
            exports,
            imports,
            metadata,
        };

        // Add to components map
        let _ = self.components.insert(id.clone(), definition);

        // Update dependency graph
        self.link_graph.add_component(id)?;

        // Update statistics
        self.stats.components_registered += 1;

        Ok(())
    }

    /// Remove a component from the linker
    pub fn remove_component(&mut self, id: &ComponentId) -> Result<()> {
        // Check if component exists
        if !self.components.contains_key(id) {
            return Err(Error::component_not_found("Component not found"));
        }

        // Check if any instances are using this component
        #[cfg(feature = "std")]
        let dependent_instances: Vec<_> = self
            .instances
            .values()
            .filter(|instance| {
                instance.component.id.as_ref().map(|s| s.as_str()) == Some(id.as_str())
            })
            .map(|instance| instance.id)
            .collect();

        #[cfg(not(feature = "std"))]
        let mut dependent_instances = Vec::new();
        for instance in self.instances.values() {
            if let (Some(comp_id), Ok(linker_id)) = (instance.component.id.as_ref(), id.as_str()) {
                if comp_id.as_str() == linker_id {
                    let _ = dependent_instances.push(instance.id);
                }
            }
        }

        if !dependent_instances.is_empty() {
            return Err(Error::runtime_execution_error(
                "Component has active instances and cannot be removed",
            ));
        }

        // Remove from components and graph
        self.components.remove(id);
        self.link_graph.remove_component(id)?;

        Ok(())
    }

    /// Instantiate a component with dependency resolution
    pub fn instantiate(
        &mut self,
        component_id: &ComponentId,
        config: Option<InstanceConfig>,
    ) -> Result<InstanceId> {
        // Find component definition and extract imports
        let imports = {
            let component = self
                .components
                .get(component_id)
                .ok_or_else(|| Error::component_not_found("Component not found"))?;
            component.imports.clone()
        };

        // Resolve dependencies
        #[cfg(feature = "std")]
        let resolved_imports = self.resolve_imports(component_id, &imports)?;
        #[cfg(not(feature = "std"))]
        let resolved_imports = {
            // Convert BoundedVec to slice for resolve_imports
            self.resolve_imports(component_id, imports.as_slice())?
        };

        // Get component again for instance creation
        let component = self
            .components
            .get(component_id)
            .ok_or_else(|| Error::component_not_found("Component not found"))?;

        // Create instance
        let instance_id = self.next_instance_id;
        self.next_instance_id += 1;

        // Create Component from definition (using new constructor)
        let mut comp = Component::new(crate::components::component::WrtComponentType::default());

        // Set the component ID
        #[cfg(feature = "std")]
        {
            comp.id = Some(component.id.clone());
        }
        #[cfg(not(feature = "std"))]
        {
            // Convert BoundedString to String for Component id
            if let Ok(id_str) = component.id.as_str() {
                comp.id = Some(alloc::string::String::from(id_str));
            }
        }

        // Create ComponentInstance
        let mut instance = ComponentInstance {
            id: instance_id,
            component: comp,
            state: crate::types::ComponentInstanceState::Initialized,
            resource_manager: None,
            memory: None,
            #[cfg(all(feature = "std", feature = "safety-critical"))]
            functions: wrt_foundation::allocator::WrtVec::new(),
            #[cfg(all(feature = "std", not(feature = "safety-critical")))]
            functions: Vec::new(),
            #[cfg(not(feature = "std"))]
            functions: BoundedVec::new(),
            #[cfg(all(feature = "std", feature = "safety-critical"))]
            imports: wrt_foundation::allocator::WrtVec::new(),
            #[cfg(all(feature = "std", not(feature = "safety-critical")))]
            imports: Vec::new(),
            #[cfg(not(feature = "std"))]
            imports: {
                let provider = safe_managed_alloc!(65536, CrateId::Component)?;
                BoundedVec::new()
            },
            #[cfg(all(feature = "std", feature = "safety-critical"))]
            exports: wrt_foundation::allocator::WrtVec::new(),
            #[cfg(all(feature = "std", not(feature = "safety-critical")))]
            exports: Vec::new(),
            #[cfg(not(feature = "std"))]
            exports: {
                let provider = safe_managed_alloc!(65536, CrateId::Component)?;
                BoundedVec::new()
            },
            #[cfg(all(feature = "std", feature = "safety-critical"))]
            resource_tables: wrt_foundation::allocator::WrtVec::new(),
            #[cfg(all(feature = "std", not(feature = "safety-critical")))]
            resource_tables: Vec::new(),
            #[cfg(not(feature = "std"))]
            resource_tables: {
                let provider = safe_managed_alloc!(65536, CrateId::Component)?;
                BoundedVec::new()
            },
            #[cfg(all(feature = "std", feature = "safety-critical"))]
            module_instances: wrt_foundation::allocator::WrtVec::new(),
            #[cfg(all(feature = "std", not(feature = "safety-critical")))]
            module_instances: Vec::new(),
            #[cfg(not(feature = "std"))]
            module_instances: {
                let provider = safe_managed_alloc!(65536, CrateId::Component)?;
                BoundedVec::new()
            },
            metadata: crate::types::ComponentMetadata::default(),
        };

        // Add resolved imports from instantiation module
        #[cfg(feature = "std")]
        for resolved in resolved_imports {
            instance.imports.push(crate::instantiation::ResolvedImport::Value(
                crate::prelude::WrtComponentValue::Unit,
            ));
        }
        #[cfg(not(feature = "std"))]
        for resolved in resolved_imports {
            let _ = instance.imports.push(crate::instantiation::ResolvedImport::Value(
                crate::prelude::WrtComponentValue::Unit,
            ));
        }

        // Transition to running state
        instance.state = crate::types::ComponentInstanceState::Running;

        // Add to instances map
        let _ = self.instances.insert(instance_id, instance);

        // Update statistics
        self.stats.instances_created += 1;

        Ok(instance_id)
    }

    /// Link all components and create instances
    pub fn link_all(&mut self) -> Result<Vec<InstanceId>> {
        let mut instance_ids = Vec::new();

        // Topological sort to determine instantiation order
        let sorted_components = self.link_graph.topological_sort()?;

        // Instantiate components in dependency order
        for component_id in sorted_components {
            let instance_id = self.instantiate(&component_id, None)?;
            let _ = instance_ids.push(instance_id);
        }

        Ok(instance_ids)
    }

    /// Get a component instance by ID
    pub fn get_instance(&self, instance_id: InstanceId) -> Option<&ComponentInstance> {
        self.instances.get(&instance_id)
    }

    /// Get a mutable component instance by ID
    pub fn get_instance_mut(&mut self, instance_id: InstanceId) -> Option<&mut ComponentInstance> {
        self.instances.get_mut(&instance_id)
    }

    /// Get linking statistics
    pub fn get_stats(&self) -> &LinkingStats {
        &self.stats
    }

    // Private helper methods

    fn parse_component_binary(
        &self,
        binary: &[u8],
    ) -> core::result::Result<(
        Vec<ComponentExport>,
        Vec<ComponentImport>,
        ComponentMetadata,
    ), Error> {
        // Simplified component parsing
        if binary.is_empty() {
            return Err(Error::runtime_execution_error("Empty component binary"));
        }

        // Create some example exports and imports based on binary content
        #[cfg(feature = "std")]
        let exports = vec![create_component_export(
            "main".to_owned(),
            ExportType::Function(crate::component_instantiation::create_function_signature(
                "main".to_owned(),
                vec![],
                vec![crate::canonical_abi::ComponentType::S32],
            )),
        )];

        #[cfg(not(feature = "std"))]
        let exports = {
            let mut exports: Vec<ComponentExport> = Vec::new();
            let mut params: Vec<crate::canonical_abi::ComponentType> = Vec::new();
            let mut results: Vec<crate::canonical_abi::ComponentType> = Vec::new();
            results.push(crate::canonical_abi::ComponentType::S32).map_err(|_| {
                Error::platform_memory_allocation_failed("Memory allocation failed")
            })?;

            let name_provider1 = safe_managed_alloc!(1024, CrateId::Component).map_err(|_| {
                Error::platform_memory_allocation_failed("Memory allocation failed")
            })?;
            let name_provider2 = safe_managed_alloc!(1024, CrateId::Component).map_err(|_| {
                Error::platform_memory_allocation_failed("Memory allocation failed")
            })?;

            // Convert params and results to Vec for create_function_signature
            // Convert to std Vec for create_function_signature which expects Vec
            #[cfg(feature = "std")]
            let params_vec: Vec<crate::canonical_abi::ComponentType> = params.iter().cloned().collect();
            #[cfg(not(feature = "std"))]
            let params_vec: alloc::vec::Vec<crate::canonical_abi::ComponentType> =
                params.iter().cloned().collect();

            #[cfg(feature = "std")]
            let results_vec: Vec<crate::canonical_abi::ComponentType> = results.iter().cloned().collect();
            #[cfg(not(feature = "std"))]
            let results_vec: alloc::vec::Vec<crate::canonical_abi::ComponentType> =
                results.iter().cloned().collect();

            #[cfg(feature = "std")]
            let name_str = "main".to_owned();
            #[cfg(not(feature = "std"))]
            let name_str = alloc::string::String::from("main");

            let signature = crate::component_instantiation::create_function_signature(
                name_str,
                params_vec,
                results_vec,
            );

            #[cfg(feature = "std")]
            let export_name = "main".to_owned();
            #[cfg(not(feature = "std"))]
            let export_name = alloc::string::String::from("main");

            exports
                .push(create_component_export(
                    export_name,
                    ExportType::Function(signature),
                ))
                .map_err(|_| {
                    Error::platform_memory_allocation_failed("Memory allocation failed")
                })?;
            exports
        };

        #[cfg(feature = "std")]
        let imports = vec![create_component_import(
            "log".to_owned(),
            "env".to_owned(),
            ImportType::Function(crate::component_instantiation::create_function_signature(
                "log".to_owned(),
                vec![crate::canonical_abi::ComponentType::String],
                vec![],
            )),
        )];

        #[cfg(not(feature = "std"))]
        let imports = {
            let mut imp_vec = Vec::new();

            let mut params: Vec<crate::canonical_abi::ComponentType> = Vec::new();
            let _ = params.push(crate::canonical_abi::ComponentType::String);

            let results: Vec<crate::canonical_abi::ComponentType> = Vec::new();

            // Convert params and results to std Vec for create_function_signature
            #[cfg(feature = "std")]
            let params_vec: Vec<crate::canonical_abi::ComponentType> = params.iter().cloned().collect();
            #[cfg(not(feature = "std"))]
            let params_vec: alloc::vec::Vec<crate::canonical_abi::ComponentType> =
                params.iter().cloned().collect();

            #[cfg(feature = "std")]
            let results_vec: Vec<crate::canonical_abi::ComponentType> = results.iter().cloned().collect();
            #[cfg(not(feature = "std"))]
            let results_vec: alloc::vec::Vec<crate::canonical_abi::ComponentType> =
                results.iter().cloned().collect();

            // Use std String for component instantiation
            let log_name = alloc::string::String::from("log");
            let env_name = alloc::string::String::from("env");
            let log_func_name = alloc::string::String::from("log");

            let _ = imp_vec.push(create_component_import(
                log_name,
                env_name,
                ImportType::Function(crate::component_instantiation::create_function_signature(
                    log_func_name,
                    params_vec,
                    results_vec,
                )),
            ));
            imp_vec
        };

        let metadata = ComponentMetadata::default();

        Ok((exports, imports, metadata))
    }

    fn resolve_imports(
        &mut self,
        component_id: &ComponentId,
        imports: &[ComponentImport],
    ) -> Result<Vec<crate::instantiation::ResolvedImport>> {
        let mut resolved = Vec::new();

        for import in imports {
            let resolution = self.resolve_single_import(component_id, import)?;
            let _ = resolved.push(resolution);
        }

        self.stats.links_resolved += resolved.len() as u32;
        Ok(resolved)
    }

    fn resolve_single_import(
        &self,
        _component_id: &ComponentId,
        import: &ComponentImport,
    ) -> Result<crate::instantiation::ResolvedImport> {
        // Find a component that exports what we need
        for (provider_id, component) in &self.components {
            for export in &component.exports {
                if self.is_compatible_import_export(import, export)? {
                    // Return a placeholder resolved import (actual resolution would be more complex)
                    return Ok(crate::instantiation::ResolvedImport::Value(
                        crate::prelude::WrtComponentValue::Unit,
                    ));
                }
            }
        }

        Err(Error::component_not_found("Component not found"))
    }

    fn is_compatible_import_export(
        &self,
        import: &ComponentImport,
        export: &ComponentExport,
    ) -> Result<bool> {
        // Check name compatibility
        if import.name != export.name {
            return Ok(false);
        }

        // Check type compatibility
        match (&import.import_type, &export.export_type) {
            (ImportType::Function(import_sig), ExportType::Function(export_sig)) => {
                Ok(self.is_compatible_function_signature(import_sig, export_sig))
            },
            (ImportType::Memory(import_mem), ExportType::Memory(export_mem)) => {
                Ok(self.is_compatible_memory_config(import_mem, export_mem))
            },
            _ => Ok(false), // Other type combinations
        }
    }

    fn is_compatible_function_signature(
        &self,
        import_sig: &FunctionSignature,
        export_sig: &FunctionSignature,
    ) -> bool {
        // Simplified compatibility check
        import_sig.params == export_sig.params && import_sig.returns == export_sig.returns
    }

    fn is_compatible_memory_config(
        &self,
        _import_mem: &crate::component_instantiation::MemoryConfig,
        _export_mem: &crate::component_instantiation::MemoryConfig,
    ) -> bool {
        // Simplified compatibility check
        true
    }
}

impl Default for LinkGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkGraph {
    /// Create a new empty link graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a component to the graph
    pub fn add_component(&mut self, component_id: ComponentId) -> Result<()> {
        // Check if component already exists
        if self.find_node_index(&component_id).is_some() {
            return Err(Error::runtime_execution_error(
                "Component already exists in graph",
            ));
        }

        let node = GraphNode {
            component_id,
            index: self.nodes.len(),
            dependencies: Vec::new(),
            dependents: Vec::new(),
        };

        let _ = self.nodes.push(node);
        Ok(())
    }

    /// Remove a component from the graph
    pub fn remove_component(&mut self, component_id: &ComponentId) -> Result<()> {
        let node_index = self.find_node_index(component_id).ok_or_else(|| {
            Error::new(
                ErrorCategory::Runtime,
                wrt_error::codes::RESOURCE_NOT_FOUND,
                "Component not found in graph",
            )
        })?;

        // Remove all edges involving this node
        self.edges.retain(|edge| edge.from != node_index && edge.to != node_index);

        // Remove the node
        self.nodes.remove(node_index);

        // Update indices in remaining nodes and edges
        #[cfg(feature = "std")]
        for node in &mut self.nodes[node_index..] {
            node.index -= 1;
        }
        #[cfg(not(feature = "std"))]
        for i in node_index..self.nodes.len() {
            self.nodes[i].index -= 1;
        }

        for edge in &mut self.edges {
            if edge.from > node_index {
                edge.from -= 1;
            }
            if edge.to > node_index {
                edge.to -= 1;
            }
        }

        Ok(())
    }

    /// Perform topological sort to determine instantiation order
    pub fn topological_sort(&self) -> Result<Vec<ComponentId>> {
        #[cfg(feature = "std")]
        {
            let mut visited = vec![false; self.nodes.len()];
            let mut temp_visited = vec![false; self.nodes.len()];
            let mut result = Vec::new();

            for i in 0..self.nodes.len() {
                if !visited[i] {
                    self.topological_sort_visit(i, &mut visited, &mut temp_visited, &mut result)?;
                }
            }

            result.reverse();
            Ok(result)
        }
        #[cfg(not(feature = "std"))]
        {
            // For no_std, create bounded vectors
            let mut visited = Vec::new();
            let mut temp_visited = Vec::new();
            let mut result = Vec::new();

            // Initialize with false values
            for _ in 0..self.nodes.len() {
                visited.push(false).map_err(|_| {
                    Error::platform_memory_allocation_failed("Memory allocation failed")
                })?;
                temp_visited.push(false).map_err(|_| {
                    Error::platform_memory_allocation_failed("Memory allocation failed")
                })?;
            }

            for i in 0..self.nodes.len() {
                if !visited[i] {
                    self.topological_sort_visit(i, &mut visited, &mut temp_visited, &mut result)?;
                }
            }

            result.reverse();
            Ok(result)
        }
    }

    fn topological_sort_visit(
        &self,
        node_index: usize,
        visited: &mut Vec<bool>,
        temp_visited: &mut Vec<bool>,
        result: &mut Vec<ComponentId>,
    ) -> Result<()> {
        if temp_visited[node_index] {
            return Err(Error::validation_error("Circular dependency detected"));
        }

        if visited[node_index] {
            return Ok(());
        }

        temp_visited[node_index] = true;

        // Visit dependencies first
        for &dep_index in &self.nodes[node_index].dependencies {
            self.topological_sort_visit(dep_index, visited, temp_visited, result)?;
        }

        temp_visited[node_index] = false;
        visited[node_index] = true;
        let _ = result.push(self.nodes[node_index].component_id.clone());

        Ok(())
    }

    fn find_node_index(&self, component_id: &ComponentId) -> Option<usize> {
        self.nodes
            .iter()
            .find(|node| &node.component_id == component_id)
            .map(|node| node.index)
    }
}

impl Default for ComponentLinker {
    fn default() -> Self {
        Self::new()
    }
}
