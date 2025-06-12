//! Bounded Infrastructure for WRT Main Crate
//!
//! This module provides bounded alternatives for main runtime collections
//! to ensure static memory allocation throughout the main WRT interface.

#![cfg_attr(not(feature = "std"), no_std)]

use wrt_foundation::{
    bounded::{BoundedVec, BoundedString},
    no_std_hashmap::BoundedHashMap,
    budget_provider::BudgetProvider,
    budget_aware_provider::{BudgetAwareProviderFactory, CrateId},
    WrtResult,
};

/// Budget-aware memory provider for main WRT (128KB)
pub type WrtProvider = BudgetProvider<131072>;

/// Maximum number of WRT engines
pub const MAX_WRT_ENGINES: usize = 16;

/// Maximum number of loaded modules
pub const MAX_LOADED_MODULES: usize = 128;

/// Maximum number of runtime configurations
pub const MAX_RUNTIME_CONFIGS: usize = 32;

/// Maximum number of engine instances
pub const MAX_ENGINE_INSTANCES: usize = 64;

/// Maximum number of linker imports
pub const MAX_LINKER_IMPORTS: usize = 1024;

/// Maximum number of linker exports
pub const MAX_LINKER_EXPORTS: usize = 1024;

/// Maximum number of stores
pub const MAX_STORES: usize = 32;

/// Maximum configuration key length
pub const MAX_CONFIG_KEY_LEN: usize = 128;

/// Maximum configuration value length
pub const MAX_CONFIG_VALUE_LEN: usize = 512;

/// Maximum module path length
pub const MAX_MODULE_PATH_LEN: usize = 512;

/// Maximum engine name length
pub const MAX_ENGINE_NAME_LEN: usize = 128;

/// Maximum number of engine features
pub const MAX_ENGINE_FEATURES: usize = 64;

/// Maximum number of wasi modules
pub const MAX_WASI_MODULES: usize = 32;

/// Maximum number of component instances
pub const MAX_COMPONENT_INSTANCES: usize = 256;

/// Bounded vector for WRT engines
pub type BoundedWrtEngineVec<T> = BoundedVec<T, MAX_WRT_ENGINES, WrtProvider>;

/// Bounded vector for loaded modules
pub type BoundedLoadedModuleVec<T> = BoundedVec<T, MAX_LOADED_MODULES, WrtProvider>;

/// Bounded vector for runtime configurations
pub type BoundedRuntimeConfigVec<T> = BoundedVec<T, MAX_RUNTIME_CONFIGS, WrtProvider>;

/// Bounded vector for engine instances
pub type BoundedEngineInstanceVec<T> = BoundedVec<T, MAX_ENGINE_INSTANCES, WrtProvider>;

/// Bounded vector for linker imports
pub type BoundedLinkerImportVec<T> = BoundedVec<T, MAX_LINKER_IMPORTS, WrtProvider>;

/// Bounded vector for linker exports
pub type BoundedLinkerExportVec<T> = BoundedVec<T, MAX_LINKER_EXPORTS, WrtProvider>;

/// Bounded vector for stores
pub type BoundedStoreVec<T> = BoundedVec<T, MAX_STORES, WrtProvider>;

/// Bounded vector for engine features
pub type BoundedEngineFeatureVec<T> = BoundedVec<T, MAX_ENGINE_FEATURES, WrtProvider>;

/// Bounded vector for WASI modules
pub type BoundedWasiModuleVec<T> = BoundedVec<T, MAX_WASI_MODULES, WrtProvider>;

/// Bounded vector for component instances
pub type BoundedComponentInstanceVec<T> = BoundedVec<T, MAX_COMPONENT_INSTANCES, WrtProvider>;

/// Bounded string for configuration keys
pub type BoundedConfigKey = BoundedString<MAX_CONFIG_KEY_LEN, WrtProvider>;

/// Bounded string for configuration values
pub type BoundedConfigValue = BoundedString<MAX_CONFIG_VALUE_LEN, WrtProvider>;

/// Bounded string for module paths
pub type BoundedModulePath = BoundedString<MAX_MODULE_PATH_LEN, WrtProvider>;

/// Bounded string for engine names
pub type BoundedEngineName = BoundedString<MAX_ENGINE_NAME_LEN, WrtProvider>;

/// Bounded map for configurations
pub type BoundedConfigMap = BoundedHashMap<
    BoundedConfigKey,
    BoundedConfigValue,
    MAX_RUNTIME_CONFIGS,
    WrtProvider
>;

/// Bounded map for loaded modules
pub type BoundedModuleMap<V> = BoundedHashMap<
    BoundedModulePath,
    V,
    MAX_LOADED_MODULES,
    WrtProvider
>;

/// Bounded map for engines
pub type BoundedEngineMap<V> = BoundedHashMap<
    BoundedEngineName,
    V,
    MAX_WRT_ENGINES,
    WrtProvider
>;

/// Bounded map for linker functions
pub type BoundedLinkerFunctionMap<V> = BoundedHashMap<
    BoundedString<MAX_CONFIG_KEY_LEN, WrtProvider>,
    V,
    MAX_LINKER_IMPORTS,
    WrtProvider
>;

/// Create a new bounded WRT engine vector
pub fn new_wrt_engine_vec<T>() -> WrtResult<BoundedWrtEngineVec<T>> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedVec::new(provider)
}

/// Create a new bounded loaded module vector
pub fn new_loaded_module_vec<T>() -> WrtResult<BoundedLoadedModuleVec<T>> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedVec::new(provider)
}

/// Create a new bounded runtime config vector
pub fn new_runtime_config_vec<T>() -> WrtResult<BoundedRuntimeConfigVec<T>> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedVec::new(provider)
}

/// Create a new bounded engine instance vector
pub fn new_engine_instance_vec<T>() -> WrtResult<BoundedEngineInstanceVec<T>> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedVec::new(provider)
}

/// Create a new bounded store vector
pub fn new_store_vec<T>() -> WrtResult<BoundedStoreVec<T>> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedVec::new(provider)
}

/// Create a new bounded configuration key
pub fn new_config_key() -> WrtResult<BoundedConfigKey> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    Ok(BoundedString::new(provider))
}

/// Create a bounded configuration key from str
pub fn bounded_config_key_from_str(s: &str) -> WrtResult<BoundedConfigKey> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedString::from_str(s, provider)
}

/// Create a new bounded configuration value
pub fn new_config_value() -> WrtResult<BoundedConfigValue> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    Ok(BoundedString::new(provider))
}

/// Create a bounded configuration value from str
pub fn bounded_config_value_from_str(s: &str) -> WrtResult<BoundedConfigValue> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedString::from_str(s, provider)
}

/// Create a new bounded module path
pub fn new_module_path() -> WrtResult<BoundedModulePath> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    Ok(BoundedString::new(provider))
}

/// Create a bounded module path from str
pub fn bounded_module_path_from_str(s: &str) -> WrtResult<BoundedModulePath> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedString::from_str(s, provider)
}

/// Create a new bounded engine name
pub fn new_engine_name() -> WrtResult<BoundedEngineName> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    Ok(BoundedString::new(provider))
}

/// Create a bounded engine name from str
pub fn bounded_engine_name_from_str(s: &str) -> WrtResult<BoundedEngineName> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedString::from_str(s, provider)
}

/// Create a new bounded configuration map
pub fn new_config_map() -> WrtResult<BoundedConfigMap> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedHashMap::new(provider)
}

/// Create a new bounded module map
pub fn new_module_map<V>() -> WrtResult<BoundedModuleMap<V>> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedHashMap::new(provider)
}

/// Create a new bounded engine map
pub fn new_engine_map<V>() -> WrtResult<BoundedEngineMap<V>> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedHashMap::new(provider)
}

/// Create a new bounded linker function map
pub fn new_linker_function_map<V>() -> WrtResult<BoundedLinkerFunctionMap<V>> {
    let provider = WrtProvider::new(CrateId::Runtime)?;
    BoundedHashMap::new(provider)
}