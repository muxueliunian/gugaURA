#![allow(non_snake_case, non_upper_case_globals)]

use std::sync::atomic::{AtomicUsize, Ordering};
use windows::{
    core::{BOOL, PCWSTR},
    Win32::Foundation::{SetLastError, ERROR_FILE_NOT_FOUND},
};

use crate::{core::Error, windows::interceptor};

use super::{ffi, log_impl};

static mut PathFileExistsW_orig: usize = 0;
static PATH_LOG_COUNT: AtomicUsize = AtomicUsize::new(0);
type PathFileExistsWFn = extern "C" fn(filename: PCWSTR) -> BOOL;
unsafe extern "C" fn PathFileExistsW(filename: PCWSTR) -> BOOL {
    let filename_str = match unsafe { filename.to_string() } {
        Ok(value) => value,
        Err(_) => {
            log_impl::raw_debug_output("[Cellar] PathFileExistsW received invalid utf-16");
            let orig_fn: PathFileExistsWFn = unsafe { std::mem::transmute(PathFileExistsW_orig) };
            return orig_fn(filename);
        }
    };

    let seen = PATH_LOG_COUNT.fetch_add(1, Ordering::Relaxed);
    if seen < 10 {
        log_impl::raw_debug_output(&format!(
            "[Cellar] PathFileExistsW path={}",
            filename_str
        ));
    }

    if filename_str.to_lowercase().ends_with(".exe.local") {
        info!("Masking .local existence: {}", filename_str);
        log_impl::raw_debug_output(&format!(
            "[Cellar] masking .local existence for {}",
            filename_str
        ));
        SetLastError(ERROR_FILE_NOT_FOUND);
        return false.into();
    }

    let orig_fn: PathFileExistsWFn = unsafe { std::mem::transmute(PathFileExistsW_orig) };
    orig_fn(filename)
}

fn init_internal() -> Result<(), Error> {
    unsafe {
        info!("Hooking PathFileExistsW");
        log_impl::raw_debug_output("[Cellar] Hooking PathFileExistsW");
        PathFileExistsW_orig =
            interceptor::hook(ffi::PathFileExistsW as usize, PathFileExistsW as usize)?;
        log_impl::raw_debug_output("[Cellar] PathFileExistsW hook installed");
    }

    Ok(())
}

pub fn init() {
    init_internal().unwrap_or_else(|e| {
        error!("Init failed: {}", e);
        log_impl::raw_debug_output(&format!("[Cellar] init failed: {}", e));
    });
}
