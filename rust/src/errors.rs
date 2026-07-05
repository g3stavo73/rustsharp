use std::{
    fmt,
    path::PathBuf,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Native,
    DotNet,
    Wasm,
}

impl fmt::Display for BackendKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native => write!(f, "native"),
            Self::DotNet => write!(f, ".NET"),
            Self::Wasm => write!(f, "WASM"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleState {
    Unloaded,
    Loaded,
    Initialized,
    Running,
    Failed,
    Unloading,
}

impl fmt::Display for ModuleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unloaded => write!(f, "Unloaded"),
            Self::Loaded => write!(f, "Loaded"),
            Self::Initialized => write!(f, "Initialized"),
            Self::Running => write!(f, "Running"),
            Self::Failed => write!(f, "Failed"),
            Self::Unloading => write!(f, "Unloading"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCode {
    AbiMismatch,
    InvalidManifest,
    LibraryLoad,
    MissingExport,
    ModuleNotFound,
    InvalidStateTransition,
    LifecycleError,
    BackendUnavailable,
    AlreadyInstalled,
    Io,
    UnsupportedFeature,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbiMismatch => write!(f, "ERR_ABI_MISMATCH"),
            Self::InvalidManifest => write!(f, "ERR_INVALID_MANIFEST"),
            Self::LibraryLoad => write!(f, "ERR_LIBRARY_LOAD"),
            Self::MissingExport => write!(f, "ERR_MISSING_EXPORT"),
            Self::ModuleNotFound => write!(f, "ERR_MODULE_NOT_FOUND"),
            Self::InvalidStateTransition => write!(f, "ERR_INVALID_STATE_TRANSITION"),
            Self::LifecycleError => write!(f, "ERR_LIFECYCLE"),
            Self::BackendUnavailable => write!(f, "ERR_BACKEND_UNAVAILABLE"),
            Self::AlreadyInstalled => write!(f, "ERR_ALREADY_INSTALLED"),
            Self::Io => write!(f, "ERR_IO"),
            Self::UnsupportedFeature => write!(f, "ERR_UNSUPPORTED_FEATURE"),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum RuntimeError {
    AbiMismatch {
        module: String,
        found: u32,
        expected: u32,
    },

    InvalidManifest {
        module: Option<String>,
        path: Option<PathBuf>,
        detail: String,
    },

    LibraryLoad {
        path: Option<PathBuf>,
        detail: String,
    },

    MissingExport {
        module: String,
        symbol: String,
    },

    ModuleNotFound {
        name: String,
    },

    InvalidStateTransition {
        module: Option<String>,
        from: ModuleState,
        to: ModuleState,
    },

    LifecycleError {
        module: Option<String>,
        hook: String,
        detail: String,
    },

    BackendUnavailable {
        backend: BackendKind,
    },

    AlreadyInstalled {
        name: String,
    },

    UnsupportedFeature {
        feature: &'static str,
    },

    Io {
        detail: String,
    },
}

impl RuntimeError {
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::AbiMismatch { .. } => ErrorCode::AbiMismatch,
            Self::InvalidManifest { .. } => ErrorCode::InvalidManifest,
            Self::LibraryLoad { .. } => ErrorCode::LibraryLoad,
            Self::MissingExport { .. } => ErrorCode::MissingExport,
            Self::ModuleNotFound { .. } => ErrorCode::ModuleNotFound,
            Self::InvalidStateTransition { .. } => ErrorCode::InvalidStateTransition,
            Self::LifecycleError { .. } => ErrorCode::LifecycleError,
            Self::BackendUnavailable { .. } => ErrorCode::BackendUnavailable,
            Self::AlreadyInstalled { .. } => ErrorCode::AlreadyInstalled,
            Self::UnsupportedFeature { .. } => ErrorCode::UnsupportedFeature,
            Self::Io { .. } => ErrorCode::Io,
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbiMismatch {
                module,
                found,
                expected,
            } => write!(
                f,
                "[{}] module '{module}' has ABI v{found}, expected v{expected}",
                self.code()
            ),

            Self::InvalidManifest {
                module,
                path,
                detail,
            } => match (module, path) {
                (Some(module), Some(path)) => write!(
                    f,
                    "[{}] invalid manifest for module '{module}' at '{}': {detail}",
                    self.code(),
                    path.display()
                ),
                (Some(module), None) => write!(
                    f,
                    "[{}] invalid manifest for module '{module}': {detail}",
                    self.code()
                ),
                (None, Some(path)) => write!(
                    f,
                    "[{}] invalid manifest at '{}': {detail}",
                    self.code(),
                    path.display()
                ),
                (None, None) => write!(f, "[{}] invalid manifest: {detail}", self.code()),
            },

            Self::LibraryLoad { path, detail } => match path {
                Some(path) => write!(
                    f,
                    "[{}] failed to load library '{}': {detail}",
                    self.code(),
                    path.display()
                ),
                None => write!(f, "[{}] failed to load library: {detail}", self.code()),
            },

            Self::MissingExport { module, symbol } => write!(
                f,
                "[{}] export '{symbol}' is missing from module '{module}'",
                self.code()
            ),

            Self::ModuleNotFound { name } => {
                write!(f, "[{}] module not found: '{name}'", self.code())
            }

            Self::InvalidStateTransition { module, from, to } => match module {
                Some(module) => write!(
                    f,
                    "[{}] invalid state transition for module '{module}': {from} → {to}",
                    self.code()
                ),
                None => write!(
                    f,
                    "[{}] invalid state transition: {from} → {to}",
                    self.code()
                ),
            },

            Self::LifecycleError {
                module,
                hook,
                detail,
            } => match module {
                Some(module) => write!(
                    f,
                    "[{}] lifecycle hook '{hook}' failed for module '{module}': {detail}",
                    self.code()
                ),
                None => write!(
                    f,
                    "[{}] lifecycle hook '{hook}' failed: {detail}",
                    self.code()
                ),
            },

            Self::BackendUnavailable { backend } => write!(
                f,
                "[{}] backend '{backend}' is not available in this build",
                self.code()
            ),

            Self::AlreadyInstalled { name } => {
                write!(f, "[{}] module '{name}' is already installed", self.code())
            }

            Self::UnsupportedFeature { feature } => {
                write!(f, "[{}] feature '{feature}' is not supported in this build", self.code())
            }

            Self::Io { detail } => write!(f, "[{}] I/O error: {detail}", self.code()),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<std::io::Error> for RuntimeError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            detail: err.to_string(),
        }
    }
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[must_use]
pub fn invalid_manifest(module: impl Into<Option<String>>, detail: impl Into<String>) -> RuntimeError {
    RuntimeError::InvalidManifest {
        module: module.into(),
        path: None,
        detail: detail.into(),
    }
}

#[must_use]
pub fn library_load(path: impl Into<Option<PathBuf>>, detail: impl Into<String>) -> RuntimeError {
    RuntimeError::LibraryLoad {
        path: path.into(),
        detail: detail.into(),
    }
}

#[must_use]
pub fn lifecycle_error(
    module: impl Into<Option<String>>,
    hook: impl Into<String>,
    detail: impl Into<String>,
) -> RuntimeError {
    RuntimeError::LifecycleError {
        module: module.into(),
        hook: hook.into(),
        detail: detail.into(),
    }
}

#[must_use]
pub fn invalid_state_transition(
    module: impl Into<Option<String>>,
    from: ModuleState,
    to: ModuleState,
) -> RuntimeError {
    RuntimeError::InvalidStateTransition {
        module: module.into(),
        from,
        to,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abi_mismatch_has_code() {
        let err = RuntimeError::AbiMismatch {
            module: "math".into(),
            found: 2,
            expected: 1,
        };
        assert_eq!(err.code(), ErrorCode::AbiMismatch);
    }

    #[test]
    fn display_includes_code() {
        let err = RuntimeError::BackendUnavailable {
            backend: BackendKind::Wasm,
        };
        assert!(err.to_string().contains("ERR_BACKEND_UNAVAILABLE"));
    }

    #[test]
    fn state_transition_display_works() {
        let err = RuntimeError::InvalidStateTransition {
            module: Some("math".into()),
            from: ModuleState::Loaded,
            to: ModuleState::Running,
        };
        assert!(err.to_string().contains("Loaded → Running"));
    }
            }
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbiMismatch(msg) =>
                write!(f, "ABI mismatch: {msg}"),
            Self::InvalidManifest(msg) =>
                write!(f, "Invalid manifest: {msg}"),
            Self::LibraryLoad(msg) =>
                write!(f, "Library load failed: {msg}"),
            Self::MissingExport { symbol, module } =>
                write!(f, "Export '{symbol}' declared in manifest is missing from '{module}'"),
            Self::ModuleNotFound(name) =>
                write!(f, "Module not found: '{name}'"),
            Self::InvalidStateTransition { from, to } =>
                write!(f, "Invalid transition: {from} → {to}"),
            Self::LifecycleError { hook, detail } =>
                write!(f, "Lifecycle hook '{hook}' failed: {detail}"),
            Self::WasmNotSupported =>
                write!(f, "WASM backend is not available in this build (enable feature 'wasm')"),
            Self::AlreadyInstalled(name) =>
                write!(f, "Module '{name}' is already installed"),
            Self::Io(msg) =>
                write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<std::io::Error> for RuntimeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}
