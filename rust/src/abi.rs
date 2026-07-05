#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub const ABI_VERSION: u32 = 1;
pub const MIN_COMPATIBLE_ABI: u32 = 1;
pub const RUNTIME_VERSION: &str = "0.2.0";

const RUNTIME_VERSION_CSTR: &[u8] = b"RustSharp 0.2.0\0";

#[no_mangle]
pub extern "C" fn rustsharp_version() -> *const c_char {
    RUNTIME_VERSION_CSTR.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn rustsharp_abi_version() -> u32 {
    ABI_VERSION
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AbiCompatibility {
    Compatible,
    TooOld { module_abi: u32, min_supported: u32 },
    TooNew { module_abi: u32, runtime_abi: u32 },
}

#[must_use]
pub fn check(module_abi: u32) -> AbiCompatibility {
    if module_abi < MIN_COMPATIBLE_ABI {
        AbiCompatibility::TooOld {
            module_abi,
            min_supported: MIN_COMPATIBLE_ABI,
        }
    } else if module_abi > ABI_VERSION {
        AbiCompatibility::TooNew {
            module_abi,
            runtime_abi: ABI_VERSION,
        }
    } else {
        AbiCompatibility::Compatible
    }
}

pub fn require_compatible(module_abi: u32) -> Result<(), String> {
    match check(module_abi) {
        AbiCompatibility::Compatible => Ok(()),
        AbiCompatibility::TooOld {
            module_abi,
            min_supported,
        } => Err(format!(
            "module ABI v{module_abi} is below runtime minimum v{min_supported}"
        )),
        AbiCompatibility::TooNew {
            module_abi,
            runtime_abi,
        } => Err(format!(
            "module ABI v{module_abi} exceeds runtime v{runtime_abi} — upgrade the runtime"
        )),
    }
}

#[no_mangle]
pub extern "C" fn rustsharp_add(a: i64, b: i64) -> i64 {
    a.saturating_add(b)
}

#[no_mangle]
pub extern "C" fn rustsharp_fibonacci(n: u32) -> i64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut prev = 0i64;
            let mut curr = 1i64;

            for _ in 2..=n {
                let next = prev.saturating_add(curr);
                prev = curr;
                curr = next;
            }

            curr
        }
    }
}

#[no_mangle]
pub extern "C" fn rustsharp_is_prime(n: u64) -> bool {
    match n {
        0 | 1 => false,
        2 | 3 => true,
        _ if n % 2 == 0 || n % 3 == 0 => false,
        _ => {
            let mut i = 5u64;
            while i.saturating_mul(i) <= n {
                if n % i == 0 || n % (i + 2) == 0 {
                    return false;
                }
                i += 6;
            }
            true
        }
    }
}

// Returns u64::MAX on error or if the limit is too large.
// Keeping the limit bounded prevents accidental memory exhaustion.
#[no_mangle]
pub extern "C" fn rustsharp_count_primes(limit: u64) -> u64 {
    const MAX_LIMIT: u64 = 100_000_000;

    if limit > MAX_LIMIT {
        return u64::MAX;
    }

    std::panic::catch_unwind(|| {
        let size = limit as usize + 1;
        if size < 2 {
            return size as u64 - 1;
        }

        let mut sieve = vec![true; size];
        sieve[0] = false;
        sieve[1] = false;

        let mut i = 2usize;
        while let Some(square) = i.checked_mul(i) {
            if square >= size {
                break;
            }

            if sieve[i] {
                let mut j = square;
                while j < size {
                    sieve[j] = false;
                    j += i;
                }
            }

            i += 1;
        }

        sieve.iter().filter(|&&is_prime| is_prime).count() as u64
    })
    .unwrap_or(u64::MAX)
}

#[no_mangle]
pub extern "C" fn rustsharp_string_reverse(input: *const c_char) -> *mut c_char {
    map_str(input, |s| s.chars().rev().collect())
}

#[no_mangle]
pub extern "C" fn rustsharp_string_to_uppercase(input: *const c_char) -> *mut c_char {
    map_str(input, |s| s.to_uppercase())
}

#[no_mangle]
pub extern "C" fn rustsharp_string_char_count(input: *const c_char) -> usize {
    c_to_string(input)
        .map(|s| s.chars().count())
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn rustsharp_string_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        drop(CString::from_raw(ptr));
    }
}

#[no_mangle]
pub extern "C" fn rustsharp_sort_i64(data: *mut i64, len: usize) {
    if data.is_null() || len == 0 {
        return;
    }

    unsafe {
        std::slice::from_raw_parts_mut(data, len).sort_unstable();
    }
}

#[no_mangle]
pub extern "C" fn rustsharp_sum_i64(data: *const i64, len: usize) -> i64 {
    fold_slice(data, len, 0i64, |acc, &x| acc.saturating_add(x))
}

#[no_mangle]
pub extern "C" fn rustsharp_max_i64(data: *const i64, len: usize) -> i64 {
    if data.is_null() || len == 0 {
        return i64::MIN;
    }

    fold_slice(data, len, i64::MIN, |acc, &x| acc.max(x))
}

fn map_str(ptr: *const c_char, f: impl FnOnce(&str) -> String) -> *mut c_char {
    let Some(input) = c_to_str(ptr) else {
        return std::ptr::null_mut();
    };

    CString::new(f(input))
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

fn fold_slice<T: Copy>(
    data: *const i64,
    len: usize,
    init: T,
    f: impl Fn(T, &i64) -> T,
) -> T {
    if data.is_null() || len == 0 {
        return init;
    }

    unsafe { std::slice::from_raw_parts(data, len) }
        .iter()
        .fold(init, f)
}

fn c_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatible_same_version() {
        assert_eq!(check(ABI_VERSION), AbiCompatibility::Compatible);
    }

    #[test]
    fn too_old_below_minimum() {
        if MIN_COMPATIBLE_ABI > 0 {
            assert!(matches!(check(0), AbiCompatibility::TooOld { .. }));
        }
    }

    #[test]
    fn too_new_above_runtime() {
        assert!(matches!(check(ABI_VERSION + 1), AbiCompatibility::TooNew { .. }));
    }

    #[test]
    fn version_string_is_valid_cstr() {
        assert_eq!(rustsharp_version(), RUNTIME_VERSION_CSTR.as_ptr() as *const c_char);
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
