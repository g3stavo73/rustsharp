use crate::abi;
use crate::error::{ModuleState, RuntimeError};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeTarget {
    Native,
    Wasm,
    Hybrid,
}

impl RuntimeTarget {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "wasm"   => Self::Wasm,
            "hybrid" => Self::Hybrid,
            _        => Self::Native,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Wasm   => "wasm",
            Self::Hybrid => "hybrid",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub abi_version: u32,
    pub min_runtime: String,
    pub runtime_target: RuntimeTarget,
    pub exports: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub on_load: Option<String>,
    pub on_unload: Option<String>,
}

impl ModuleManifest {
    pub fn from_toml_str(src: &str) -> Result<Self, RuntimeError> {
        let kv: HashMap<String, String> = src
            .lines()
            .filter(|l| !l.trim_start().starts_with('#') && l.contains('='))
            .filter_map(|l| {
                let mut parts = l.splitn(2, '=');
                let k = parts.next()?.trim().trim_matches('"').to_string();
                let v = parts.next()?.trim().trim_matches('"').to_string();
                Some((k, v))
            })
            .collect();

        let name    = kv.get("name")   .cloned().ok_or_else(|| RuntimeError::InvalidManifest("missing 'name'".into()))?;
        let version = kv.get("version").cloned().ok_or_else(|| RuntimeError::InvalidManifest("missing 'version'".into()))?;

        let abi_str = kv.get("version").and_then(|_| kv.get("abi_version")).cloned()
            .or_else(|| kv.get("version").cloned())
            .unwrap_or_else(|| "1".into());
        let abi_str = kv.get("abi_version").cloned().unwrap_or(abi_str);
        let abi_version: u32 = abi_str.parse().map_err(|_|
            RuntimeError::InvalidManifest(format!("abi_version '{abi_str}' is not a number"))
        )?;

        abi::require_compatible(abi_version).map_err(RuntimeError::AbiMismatch)?;

        Ok(Self {
            description:    kv.get("description").cloned().unwrap_or_default(),
            min_runtime:    kv.get("min_runtime").cloned().unwrap_or_else(|| "0.1.0".into()),
            runtime_target: RuntimeTarget::from_str(kv.get("target").map(String::as_str).unwrap_or("native")),
            exports:        kv.get("symbols").map(|s| {
                s.trim_matches(|c| c == '[' || c == ']')
                 .split(',')
                 .map(|e| e.trim().trim_matches('"').to_string())
                 .filter(|e| !e.is_empty())
                 .collect()
            }).unwrap_or_default(),
            on_load:    kv.get("on_load").cloned(),
            on_unload:  kv.get("on_unload").cloned(),
            dependencies: HashMap::new(),
            name, version, abi_version,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RshmModule {
    pub manifest:  ModuleManifest,
    pub state:     ModuleState,
    pub install_path: Option<String>,
}

impl RshmModule {
    pub fn new(manifest: ModuleManifest) -> Self {
        Self { manifest, state: ModuleState::Unloaded, install_path: None }
    }

    pub fn id(&self) -> String {
        format!("{}@{}", self.manifest.name, self.manifest.version)
    }

    pub fn transition(&mut self, next: ModuleState) -> Result<(), RuntimeError> {
        use ModuleState::*;
        let allowed = match &self.state {
            Unloaded    => matches!(next, Loaded),
            Loaded      => matches!(next, Initialized | Unloaded),
            Initialized => matches!(next, Running | Unloading),
            Running     => matches!(next, Unloading),
            Unloading   => matches!(next, Unloaded),
        };

        if allowed {
            self.state = next;
            Ok(())
        } else {
            Err(RuntimeError::InvalidStateTransition {
                from: self.state.clone(),
                to:   next,
            })
        }
    }
          } 
