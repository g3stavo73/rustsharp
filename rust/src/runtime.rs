use crate::abi;
use crate::error::{ModuleState, RuntimeError};
use crate::module::{ModuleManifest, RshmModule, RuntimeTarget};
use std::collections::HashMap;

pub struct RustSharpRuntime {
    modules: HashMap<String, RshmModule>,
}

impl RustSharpRuntime {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            modules: HashMap::with_capacity(capacity),
        }
    }

    pub fn register(&mut self, manifest: ModuleManifest) -> Result<String, RuntimeError> {
        manifest.validate_exports()?;

        let id = manifest.id();

        if self.modules.contains_key(&id) {
            return Err(RuntimeError::AlreadyInstalled(id));
        }

        self.modules.insert(id.clone(), RshmModule::new(manifest));
        Ok(id)
    }

    pub fn has_module(&self, id: &str) -> bool {
        self.modules.contains_key(id)
    }

    pub fn list_modules(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.modules.keys().cloned().collect();
        ids.sort_unstable();
        ids
    }

    fn module_mut(&mut self, id: &str) -> Result<&mut RshmModule, RuntimeError> {
        self.modules
            .get_mut(id)
            .ok_or_else(|| RuntimeError::ModuleNotFound(id.to_string()))
    }

    fn module(&self, id: &str) -> Result<&RshmModule, RuntimeError> {
        self.modules
            .get(id)
            .ok_or_else(|| RuntimeError::ModuleNotFound(id.to_string()))
    }

    fn mark_failed(&mut self, id: &str) {
        if let Some(module) = self.modules.get_mut(id) {
            module.mark_failed();
        }
    }

    fn ensure_target_supported(module: &RshmModule) -> Result<(), RuntimeError> {
        match module.manifest.runtime_target {
            RuntimeTarget::Native | RuntimeTarget::Hybrid => Ok(()),
            RuntimeTarget::Wasm => {
                #[cfg(feature = "wasm")]
                {
                    Ok(())
                }
                #[cfg(not(feature = "wasm"))]
                {
                    Err(RuntimeError::WasmNotSupported)
                }
            }
        }
    }

    pub fn load(&mut self, id: &str) -> Result<(), RuntimeError> {
        let module = self.module_mut(id)?;
        Self::ensure_target_supported(module)?;
        module.transition(ModuleState::Loaded)
    }

    pub fn init(&mut self, id: &str) -> Result<(), RuntimeError> {
        let module = self.module_mut(id)?;

        if module.state != ModuleState::Loaded {
            return Err(RuntimeError::InvalidStateTransition {
                module: Some(module.id()),
                from: module.state.clone(),
                to: ModuleState::Initialized,
            });
        }

        if let Some(hook) = module.manifest.on_load.as_deref() {
            if hook.trim().is_empty() {
                return Err(RuntimeError::LifecycleError {
                    module: Some(module.id()),
                    hook: "on_load".into(),
                    detail: "empty hook".into(),
                });
            }
        }

        module.transition(ModuleState::Initialized)
    }

    pub fn start(&mut self, id: &str) -> Result<(), RuntimeError> {
        let module = self.module_mut(id)?;

        if module.state != ModuleState::Initialized {
            return Err(RuntimeError::InvalidStateTransition {
                module: Some(module.id()),
                from: module.state.clone(),
                to: ModuleState::Running,
            });
        }

        module.transition(ModuleState::Running)
    }

    pub fn bring_up(&mut self, id: &str) -> Result<(), RuntimeError> {
        if let Err(e) = self.load(id) {
            self.mark_failed(id);
            return Err(e);
        }

        if let Err(e) = self.init(id) {
            self.mark_failed(id);
            return Err(e);
        }

        if let Err(e) = self.start(id) {
            self.mark_failed(id);
            return Err(e);
        }

        Ok(())
    }

    pub fn unload(&mut self, id: &str) -> Result<(), RuntimeError> {
        let module = self.module_mut(id)?;

        if module.state == ModuleState::Unloaded {
            return Ok(());
        }

        if let Some(hook) = module.manifest.on_unload.as_deref() {
            if hook.trim().is_empty() {
                return Err(RuntimeError::LifecycleError {
                    module: Some(module.id()),
                    hook: "on_unload".into(),
                    detail: "empty hook".into(),
                });
            }
        }

        if module.state != ModuleState::Unloading {
            module.transition(ModuleState::Unloading)?;
        }

        module.transition(ModuleState::Unloaded)
    }

    pub fn unload_all(&mut self) -> Result<(), RuntimeError> {
        let ids: Vec<String> = self.modules.keys().cloned().collect();

        for id in ids {
            let _ = self.unload(&id);
        }

        Ok(())
    }

    pub fn module_state(&self, id: &str) -> Result<ModuleState, RuntimeError> {
        Ok(self.module(id)?.state.clone())
    }

    pub fn is_running(&self, id: &str) -> Result<bool, RuntimeError> {
        Ok(self.module_state(id)? == ModuleState::Running)
    }

    pub fn validate_abi(&self, abi_version: u32) -> Result<(), RuntimeError> {
        abi::require_compatible(abi_version).map_err(|_| RuntimeError::AbiMismatch {
            module: "unknown".into(),
            found: abi_version,
            expected: abi::ABI_VERSION,
        })
    }
}

impl Default for RustSharpRuntime {
    fn default() -> Self {
        Self::new()
    }
              }
