use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use syn::{parse_file, Attribute, Item, ItemImpl, ItemTrait, ItemStruct, ItemEnum};
use quote::quote;
use serde::{Deserialize, Serialize};

/// Architecture documentation generator
/// 
/// This module extracts architectural information from code annotations
/// and generates PlantUML diagrams and documentation.

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchitecturalComponent {
    pub name: String,
    pub category: String,
    pub file_path: String,
    pub line_number: usize,
    pub interfaces: Vec<String>,
    pub states: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateMachine {
    pub name: String,
    pub initial_state: String,
    pub states: Vec<State>,
    pub transitions: Vec<Transition>,
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub name: String,
    pub description: Option<String>,
    pub entry_actions: Vec<String>,
    pub exit_actions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub trigger: String,
    pub guard: Option<String>,
    pub action: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InterfaceContract {
    pub name: String,
    pub category: String,
    pub invariants: Vec<String>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub complexity: Vec<(String, String)>,
    pub file_path: String,
}

pub struct ArchitectureExtractor {
    components: Vec<ArchitecturalComponent>,
    state_machines: Vec<StateMachine>,
    interfaces: Vec<InterfaceContract>,
}

impl ArchitectureExtractor {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            state_machines: Vec::new(),
            interfaces: Vec::new(),
        }
    }

    /// Extract architectural information from a Rust source file
    pub fn extract_from_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        let syntax = parse_file(&content)
            .with_context(|| format!("Failed to parse file: {}", path.display()))?;
        
        for item in syntax.items {
            match item {
                Item::Struct(item_struct) => {
                    self.extract_from_struct(&item_struct, path)?;
                }
                Item::Trait(item_trait) => {
                    self.extract_from_trait(&item_trait, path)?;
                }
                Item::Impl(item_impl) => {
                    self.extract_from_impl(&item_impl, path)?;
                }
                Item::Enum(item_enum) => {
                    self.extract_from_enum(&item_enum, path)?;
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    fn extract_from_struct(&mut self, item: &ItemStruct, path: &Path) -> Result<()> {
        // Look for #[derive(ArchitecturalComponent)] or #[arch_component(...)]
        for attr in &item.attrs {
            if self.is_arch_component_attr(attr) {
                let component = self.parse_arch_component(attr, &item.ident.to_string(), path)?;
                self.components.push(component);
            }
        }
        Ok(())
    }

    fn extract_from_trait(&mut self, item: &ItemTrait, path: &Path) -> Result<()> {
        // Look for #[interface_contract(...)]
        for attr in &item.attrs {
            if self.is_interface_contract_attr(attr) {
                let contract = self.parse_interface_contract(attr, &item.ident.to_string(), path)?;
                self.interfaces.push(contract);
            }
        }
        Ok(())
    }

    fn extract_from_impl(&mut self, item: &ItemImpl, path: &Path) -> Result<()> {
        // Look for #[arch_state_machine(...)]
        for attr in &item.attrs {
            if self.is_state_machine_attr(attr) {
                let state_machine = self.parse_state_machine(attr, path)?;
                self.state_machines.push(state_machine);
            }
        }
        Ok(())
    }

    fn extract_from_enum(&mut self, item: &ItemEnum, path: &Path) -> Result<()> {
        // Look for state enums with #[arch_states]
        for attr in &item.attrs {
            if self.is_arch_states_attr(attr) {
                // Extract state definitions from enum variants
                let states: Vec<State> = item.variants.iter().map(|variant| {
                    State {
                        name: variant.ident.to_string(),
                        description: self.extract_doc_comment(&variant.attrs),
                        entry_actions: Vec::new(),
                        exit_actions: Vec::new(),
                    }
                }).collect();
                
                // Store states for later use in state machine definitions
                // This is simplified - in practice, you'd need to associate these with a state machine
            }
        }
        Ok(())
    }

    fn is_arch_component_attr(&self, attr: &Attribute) -> bool {
        attr.path.is_ident("arch_component") || 
        attr.path.is_ident("derive") && attr.tokens.to_string().contains("ArchitecturalComponent")
    }

    fn is_interface_contract_attr(&self, attr: &Attribute) -> bool {
        attr.path.is_ident("interface_contract")
    }

    fn is_state_machine_attr(&self, attr: &Attribute) -> bool {
        attr.path.is_ident("arch_state_machine")
    }

    fn is_arch_states_attr(&self, attr: &Attribute) -> bool {
        attr.path.is_ident("arch_states")
    }

    fn parse_arch_component(&self, attr: &Attribute, name: &str, path: &Path) -> Result<ArchitecturalComponent> {
        // This is a simplified parser - in practice, you'd use syn's parsing utilities
        // to properly parse the attribute arguments
        Ok(ArchitecturalComponent {
            name: name.to_string(),
            category: "Core".to_string(), // Would be extracted from attr
            file_path: path.to_string_lossy().to_string(),
            line_number: 0, // Would be extracted from span
            interfaces: vec![], // Would be extracted from attr
            states: vec![], // Would be extracted from attr
            description: self.extract_doc_comment(&[]),
        })
    }

    fn parse_interface_contract(&self, attr: &Attribute, name: &str, path: &Path) -> Result<InterfaceContract> {
        // Simplified parser
        Ok(InterfaceContract {
            name: name.to_string(),
            category: "Foundation".to_string(),
            invariants: vec![],
            preconditions: vec![],
            postconditions: vec![],
            complexity: vec![],
            file_path: path.to_string_lossy().to_string(),
        })
    }

    fn parse_state_machine(&self, attr: &Attribute, path: &Path) -> Result<StateMachine> {
        // Simplified parser
        Ok(StateMachine {
            name: "ComponentLifecycle".to_string(),
            initial_state: "Loaded".to_string(),
            states: vec![],
            transitions: vec![],
            file_path: path.to_string_lossy().to_string(),
        })
    }

    fn extract_doc_comment(&self, attrs: &[Attribute]) -> Option<String> {
        let mut doc_lines = Vec::new();
        for attr in attrs {
            if attr.path.is_ident("doc") {
                if let Ok(syn::Meta::NameValue(meta)) = attr.parse_meta() {
                    if let syn::Lit::Str(lit) = meta.lit {
                        doc_lines.push(lit.value());
                    }
                }
            }
        }
        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join("\n"))
        }
    }

    /// Generate PlantUML diagrams from extracted architecture
    pub fn generate_plantuml(&self, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir)?;
        
        // Generate component diagrams
        for component in &self.components {
            let diagram = self.generate_component_diagram(component);
            let filename = format!("component_{}.puml", component.name.to_lowercase());
            let path = output_dir.join(&filename);
            fs::write(&path, diagram)?;
        }
        
        // Generate state machine diagrams
        for state_machine in &self.state_machines {
            let diagram = self.generate_state_machine_diagram(state_machine);
            let filename = format!("state_machine_{}.puml", state_machine.name.to_lowercase());
            let path = output_dir.join(&filename);
            fs::write(&path, diagram)?;
        }
        
        // Generate interface diagrams
        for interface in &self.interfaces {
            let diagram = self.generate_interface_diagram(interface);
            let filename = format!("interface_{}.puml", interface.name.to_lowercase());
            let path = output_dir.join(&filename);
            fs::write(&path, diagram)?;
        }
        
        Ok(())
    }

    fn generate_component_diagram(&self, component: &ArchitecturalComponent) -> String {
        let mut diagram = String::new();
        diagram.push_str("@startuml\n");
        diagram.push_str("!include _common.puml\n\n");
        
        diagram.push_str(&format!("component \"{}\" as {} {{\n", component.name, component.name));
        
        if let Some(desc) = &component.description {
            diagram.push_str(&format!("  note top: {}\n", desc));
        }
        
        for interface in &component.interfaces {
            diagram.push_str(&format!("  interface {}\n", interface));
        }
        
        if !component.states.is_empty() {
            diagram.push_str("  \n");
            diagram.push_str("  state LifecycleStates {\n");
            for state in &component.states {
                diagram.push_str(&format!("    state {}\n", state));
            }
            diagram.push_str("  }\n");
        }
        
        diagram.push_str("}\n");
        diagram.push_str("@enduml\n");
        
        diagram
    }

    fn generate_state_machine_diagram(&self, state_machine: &StateMachine) -> String {
        let mut diagram = String::new();
        diagram.push_str("@startuml\n");
        diagram.push_str("!include _common.puml\n\n");
        
        diagram.push_str(&format!("[*] --> {}\n\n", state_machine.initial_state));
        
        // Define states
        for state in &state_machine.states {
            diagram.push_str(&format!("state {} {{\n", state.name));
            if let Some(desc) = &state.description {
                diagram.push_str(&format!("  {} : {}\n", state.name, desc));
            }
            for action in &state.entry_actions {
                diagram.push_str(&format!("  {} : entry / {}\n", state.name, action));
            }
            for action in &state.exit_actions {
                diagram.push_str(&format!("  {} : exit / {}\n", state.name, action));
            }
            diagram.push_str("}\n\n");
        }
        
        // Define transitions
        for transition in &state_machine.transitions {
            let mut trans_str = format!("{} --> {}", transition.from, transition.to);
            
            if let Some(guard) = &transition.guard {
                trans_str.push_str(&format!(" : {} [{}]", transition.trigger, guard));
            } else {
                trans_str.push_str(&format!(" : {}", transition.trigger));
            }
            
            if let Some(action) = &transition.action {
                trans_str.push_str(&format!(" / {}", action));
            }
            
            diagram.push_str(&format!("{}\n", trans_str));
        }
        
        diagram.push_str("\n@enduml\n");
        
        diagram
    }

    fn generate_interface_diagram(&self, interface: &InterfaceContract) -> String {
        let mut diagram = String::new();
        diagram.push_str("@startuml\n");
        diagram.push_str("!include _common.puml\n\n");
        
        diagram.push_str(&format!("interface {} {{\n", interface.name));
        
        // Show methods with complexity annotations
        for (method, complexity) in &interface.complexity {
            diagram.push_str(&format!("  +{} : {}\n", method, complexity));
        }
        
        diagram.push_str("}\n\n");
        
        // Add contract notes
        if !interface.invariants.is_empty() {
            diagram.push_str(&format!("note right of {}\n", interface.name));
            diagram.push_str("  <b>Invariants:</b>\n");
            for invariant in &interface.invariants {
                diagram.push_str(&format!("  * {}\n", invariant));
            }
            diagram.push_str("end note\n");
        }
        
        diagram.push_str("@enduml\n");
        
        diagram
    }

    /// Generate documentation validation tests
    pub fn generate_validation_tests(&self, output_path: &Path) -> Result<()> {
        let mut test_content = String::new();
        
        test_content.push_str("// Auto-generated architecture validation tests\n");
        test_content.push_str("// DO NOT EDIT - Generated by xtask architecture-doc-generator\n\n");
        
        test_content.push_str("#[cfg(test)]\n");
        test_content.push_str("mod architecture_validation {\n");
        test_content.push_str("    use super::*;\n\n");
        
        // Generate component validation tests
        for component in &self.components {
            test_content.push_str(&format!(
                "    #[test]\n    fn validate_component_{}() {{\n",
                component.name.to_lowercase()
            ));
            test_content.push_str(&format!(
                "        // Verify component {} is documented\n",
                component.name
            ));
            test_content.push_str(&format!(
                "        assert!(component_documented(\"{}\"));\n",
                component.name
            ));
            test_content.push_str("    }\n\n");
        }
        
        // Generate interface validation tests
        for interface in &self.interfaces {
            test_content.push_str(&format!(
                "    #[test]\n    fn validate_interface_{}() {{\n",
                interface.name.to_lowercase()
            ));
            test_content.push_str(&format!(
                "        // Verify interface {} contracts are documented\n",
                interface.name
            ));
            test_content.push_str(&format!(
                "        assert!(interface_contracts_documented(\"{}\"));\n",
                interface.name
            ));
            test_content.push_str("    }\n\n");
        }
        
        test_content.push_str("}\n");
        
        fs::write(output_path, test_content)?;
        
        Ok(())
    }
}

/// Extract architecture from entire codebase
pub fn extract_architecture(workspace_root: &Path) -> Result<ArchitectureExtractor> {
    let mut extractor = ArchitectureExtractor::new();
    
    // Walk through all Rust source files
    for entry in walkdir::WalkDir::new(workspace_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && 
           entry.path().extension().map_or(false, |ext| ext == "rs") &&
           !entry.path().to_string_lossy().contains("target/") {
            extractor.extract_from_file(entry.path())?;
        }
    }
    
    Ok(extractor)
}

/// Generate all architecture documentation
pub fn generate_architecture_docs(workspace_root: &Path, output_dir: &Path) -> Result<()> {
    println!("Extracting architecture from codebase...");
    let extractor = extract_architecture(workspace_root)?;
    
    println!("Found {} components, {} state machines, {} interfaces",
        extractor.components.len(),
        extractor.state_machines.len(),
        extractor.interfaces.len()
    );
    
    println!("Generating PlantUML diagrams...");
    let diagrams_dir = output_dir.join("diagrams");
    extractor.generate_plantuml(&diagrams_dir)?;
    
    println!("Generating validation tests...");
    let tests_path = output_dir.join("architecture_validation_tests.rs");
    extractor.generate_validation_tests(&tests_path)?;
    
    println!("Architecture documentation generated successfully!");
    
    Ok(())
}