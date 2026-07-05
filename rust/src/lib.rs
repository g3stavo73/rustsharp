pub mod abi;
pub mod error;
pub mod module;
pub mod runtime;

mod ffi {
    pub mod math;
    pub mod str_ops;
    pub mod coll;
}

pub use ffi::coll::*;
pub use ffi::math::*;
pub use ffi::str_ops::*;
pub use abi::{rustsharp_abi_version, rustsharp_version};
pub use runtime::rustsharp_runtime_module_count;            true
        }
    }
}


#[no_mangle]
pub extern "C" fn rustsharp_count_primes(limit: u64) -> u64 {
    if limit > 100_000_000 { return u64::MAX; }
    std::panic::catch_unwind(|| {
        let size = limit as usize + 1;
        let mut sieve = vec![true; size];
        sieve[0] = false;
        if limit > 0 { sieve[1] = false; }
        let mut i = 2usize;
        while let Some(sq) = i.checked_mul(i) {
            if sq >= size { break; }
            if sieve[i] {
                let mut j = sq;
                while j < size { sieve[j] = false; j += i; }
            }
            i += 1;
        }
        sieve.iter().filter(|&&b| b).count() as u64
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
    if input.is_null() { return 0; }
    unsafe { CStr::from_ptr(input) }.to_str().map(|s| s.chars().count()).unwrap_or(0)
}


#[no_mangle]
pub extern "C" fn rustsharp_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)); }
    }
}

#[no_mangle]
pub extern "C" fn rustsharp_sort_i64(data: *mut i64, len: usize) {
    if !data.is_null() && len > 0 {
        unsafe { std::slice::from_raw_parts_mut(data, len) }.sort_unstable();
    }
}

#[no_mangle]
pub extern "C" fn rustsharp_sum_i64(data: *const i64, len: usize) -> i64 {
    fold_slice(data, len, 0i64, |acc, &x| acc.saturating_add(x))
}

#[no_mangle]
pub extern "C" fn rustsharp_max_i64(data: *const i64, len: usize) -> i64 {
    fold_slice(data, len, i64::MIN, |acc, &x| acc.max(x))
}

fn map_str(ptr: *const c_char, f: impl Fn(&str) -> String) -> *mut c_char {
    if ptr.is_null() { return std::ptr::null_mut(); }
    let Ok(s) = (unsafe { CStr::from_ptr(ptr) }).to_str() else { return std::ptr::null_mut(); };
    CString::new(f(s)).map(CString::into_raw).unwrap_or(std::ptr::null_mut())
}

fn fold_slice<T: Copy>(data: *const i64, len: usize, init: T, f: impl Fn(T, &i64) -> T) -> T {
    if data.is_null() || len == 0 { return init; }
    unsafe { std::slice::from_raw_parts(data, len) }.iter().fold(init, f)
          }
