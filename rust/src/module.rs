use crate::abi::{self, AbiCompatibility};
use crate::error::{ModuleState, RuntimeError};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeTarget {
    Native,
    Wasm,
    Hybrid,
}

impl Default for RuntimeTarget {
    fn default() -> Self {
        Self::Native
    }
}

impl FromStr for RuntimeTarget {
    type Err = RuntimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "native" => Ok(Self::Native),
            "wasm" => Ok(Self::Wasm),
            "hybrid" => Ok(Self::Hybrid),
            other => Err(RuntimeError::InvalidManifest {
                module: None,
                path: None,
                detail: format!("invalid runtime target '{other}'"),
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawPackage {
    name: String,
    version: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawRuntime {
    abi_version: u32,
    #[serde(default)]
    min_runtime: String,
    #[serde(default)]
    target: RuntimeTarget,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawExports {
    #[serde(default)]
    symbols: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawLifecycle {
    #[serde(default)]
    on_load: Option<String>,
    #[serde(default)]
    on_unload: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawModuleManifest {
    package: RawPackage,
    runtime: RawRuntime,
    #[serde(default)]
    exports: RawExports,
    #[serde(default)]
    lifecycle: RawLifecycle,
    #[serde(default)]
    dependencies: HashMap<String, String>,
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
    pub manifest_path: Option<PathBuf>,
}

impl ModuleManifest {
    pub fn from_toml_str(src: &str) -> Result<Self, RuntimeError> {
        let raw: RawModuleManifest = toml::from_str(src).map_err(|e| RuntimeError::InvalidManifest {
            module: None,
            path: None,
            detail: e.to_string(),
        })?;

        match abi::check(raw.runtime.abi_version) {
            AbiCompatibility::Compatible => {}
            AbiCompatibility::TooOld { .. } | AbiCompatibility::TooNew { .. } => {
                return Err(RuntimeError::AbiMismatch {
                    module: raw.package.name.clone(),
                    found: raw.runtime.abi_version,
                    expected: abi::ABI_VERSION,
                });
            }
        }

        Ok(Self {
            name: raw.package.name,
            version: raw.package.version,
            description: raw.package.description,
            abi_version: raw.runtime.abi_version,
            min_runtime: raw.runtime.min_runtime,
            runtime_target: raw.runtime.target,
            exports: raw.exports.symbols,
            dependencies: raw.dependencies,
            on_load: raw.lifecycle.on_load,
            on_unload: raw.lifecycle.on_unload,
            manifest_path: None,
        })
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, RuntimeError> {
        let path = path.as_ref();
        let src = fs::read_to_string(path).map_err(|e| RuntimeError::InvalidManifest {
            module: None,
            path: Some(path.to_path_buf()),
            detail: e.to_string(),
        })?;

        let mut manifest = Self::from_toml_str(&src)?;
        manifest.manifest_path = Some(path.to_path_buf());
        Ok(manifest)
    }

    pub fn id(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    pub fn is_compatible(&self) -> bool {
        matches!(abi::check(self.abi_version), AbiCompatibility::Compatible)
    }

    pub fn validate_exports(&self) -> Result<(), RuntimeError> {
        if self.name.trim().is_empty() {
            return Err(RuntimeError::InvalidManifest {
                module: None,
                path: self.manifest_path.clone(),
                detail: "package.name cannot be empty".into(),
            });
        }

        if self.version.trim().is_empty() {
            return Err(RuntimeError::InvalidManifest {
                module: Some(self.name.clone()),
                path: self.manifest_path.clone(),
                detail: "package.version cannot be empty".into(),
            });
        }

        if self.exports.is_empty() {
            return Err(RuntimeError::InvalidManifest {
                module: Some(self.name.clone()),
                path: self.manifest_path.clone(),
                detail: "exports.symbols cannot be empty".into(),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RshmModule {
    pub manifest: ModuleManifest,
    pub state: ModuleState,
    pub install_path: Option<PathBuf>,
}

impl RshmModule {
    pub fn new(manifest: ModuleManifest) -> Self {
        Self {
            manifest,
            state: ModuleState::Unloaded,
            install_path: None,
        }
    }

    pub fn id(&self) -> String {
        self.manifest.id()
    }

    pub fn with_install_path(mut self, path: impl AsRef<Path>) -> Self {
        self.install_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn transition(&mut self, next: ModuleState) -> Result<(), RuntimeError> {
        let current = self.state.clone();

        let allowed = matches!(
            (current.clone(), next.clone()),
            (ModuleState::Unloaded, ModuleState::Loaded)
                | (ModuleState::Loaded, ModuleState::Initialized)
                | (ModuleState::Loaded, ModuleState::Unloaded)
                | (ModuleState::Initialized, ModuleState::Running)
                | (ModuleState::Initialized, ModuleState::Unloading)
                | (ModuleState::Running, ModuleState::Unloading)
                | (ModuleState::Unloading, ModuleState::Unloaded)
                | (ModuleState::Failed, ModuleState::Unloading)
                | (ModuleState::Failed, ModuleState::Unloaded)
        );

        if allowed {
            self.state = next;
            Ok(())
        } else {
            Err(RuntimeError::InvalidStateTransition {
                module: Some(self.id()),
                from: current,
                to: next,
            })
        }
    }

    pub fn mark_failed(&mut self) {
        self.state = ModuleState::Failed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manifest() {
        let src = r#"
[package]
name = "math"
version = "0.1.0"
description = "math module"

[runtime]
abi_version = 1
target = "native"
min_runtime = "0.2.0"

[exports]
symbols = ["rustsharp_add", "rustsharp_fibonacci"]

[lifecycle]
on_load = "rshm_math_init"
on_unload = "rshm_math_destroy"

[dependencies]
serde = "1.0"
"#;

        let manifest = ModuleManifest::from_toml_str(src).unwrap();
        assert_eq!(manifest.name, "math");
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.abi_version, 1);
        assert_eq!(manifest.exports.len(), 2);
    }

    #[test]
    fn state_transition_works() {
        let manifest = ModuleManifest {
            name: "math".into(),
            version: "0.1.0".into(),
            description: String::new(),
            abi_version: 1,
            min_runtime: "0.2.0".into(),
            runtime_target: RuntimeTarget::Native,
            exports: vec!["rustsharp_add".into()],
            dependencies: HashMap::new(),
            on_load: None,
            on_unload: None,
            manifest_path: None,
        };

        let mut module = RshmModule::new(manifest);
        assert!(module.transition(ModuleState::Loaded).is_ok());
        assert!(module.transition(ModuleState::Initialized).is_ok());
        assert!(module.transition(ModuleState::Running).is_ok());
    }
                       }    pub runtime_target: RuntimeTarget,
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
