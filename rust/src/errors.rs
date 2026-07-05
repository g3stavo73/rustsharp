use std::fmt;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    AbiMismatch(String),
    InvalidManifest(String),
    LibraryLoad(String),
    MissingExport { symbol: String, module: String },
    ModuleNotFound(String),
    InvalidStateTransition { from: ModuleState, to: ModuleState },
    LifecycleError { hook: String, detail: String },
    WasmNotSupported,
    AlreadyInstalled(String),
    Io(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleState {
    Unloaded,
    Loaded,
    Initialized,
    Running,
    Unloading,
}

impl fmt::Display for ModuleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unloaded    => write!(f, "Unloaded"),
            Self::Loaded      => write!(f, "Loaded"),
            Self::Initialized => write!(f, "Initialized"),
            Self::Running     => write!(f, "Running"),
            Self::Unloading   => write!(f, "Unloading"),
        }
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
