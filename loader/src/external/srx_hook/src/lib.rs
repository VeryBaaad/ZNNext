/*
 * SPDX-License-Identifier: GPL-3.0-or-later AND LicenseRef-DCLPL-2.0
 *
 * This file is part of ZNNext.
 *
 * ZNNext is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * ZNNext is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with ZNNext. If not, see <https://www.gnu.org/licenses/>.
 *
 * Copyright (C) 2026 VeryBaaad <verybaaad@outlook.com>
 */

use std::collections::HashMap;
use std::ffi::{CStr, c_char, c_void};
use std::ptr;
use std::sync::{Mutex, MutexGuard, OnceLock};

use srx_hook::{HookMode, SrxHookErrno};

struct OriginalCapture {
    original: *mut c_void,
    error_status: i32,
}

fn capture_registry() -> &'static Mutex<HashMap<u64, usize>> {
    static CAPTURES: OnceLock<Mutex<HashMap<u64, usize>>> = OnceLock::new();
    CAPTURES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_captures() -> MutexGuard<'static, HashMap<u64, usize>> {
    capture_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn insert_capture(stub: u64, capture: *mut OriginalCapture) {
    lock_captures().insert(stub, capture as usize);
}

fn capture_for(stub: u64) -> Option<*mut OriginalCapture> {
    lock_captures()
        .get(&stub)
        .copied()
        .map(|value| value as *mut OriginalCapture)
}

unsafe fn release_capture(stub: u64) {
    if let Some(value) = lock_captures().remove(&stub) {
        drop(unsafe { Box::from_raw(value as *mut OriginalCapture) });
    }
}

unsafe extern "C" fn capture_original(
    _task_stub: u64,
    status_code: i32,
    _caller_path_name: *const c_char,
    _sym_name: *const c_char,
    _new_func: *mut c_void,
    prev_func: *mut c_void,
    arg: *mut c_void,
) {
    if arg.is_null() {
        return;
    }
    let capture = unsafe { &mut *(arg as *mut OriginalCapture) };
    if status_code == SrxHookErrno::Ok.as_i32() {
        if capture.original.is_null() {
            capture.original = prev_func;
        }
        return;
    }
    if status_code != SrxHookErrno::OrigAddr.as_i32() && capture.error_status == 0 {
        capture.error_status = status_code;
    }
}

unsafe fn c_string_to_str<'a>(value: *const c_char) -> Option<&'a str> {
    if value.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(value) }.to_str().ok()
}

#[unsafe(no_mangle)]
pub extern "C" fn srx_hook_ffi_init() -> i32 {
    srx_hook::init(HookMode::Manual, false).as_i32()
}

/// SAFETY: export
#[unsafe(no_mangle)]
pub unsafe extern "C" fn srx_hook_ffi_hook(
    caller_path: *const c_char,
    symbol: *const c_char,
    replacement: *mut c_void,
    original: *mut *mut c_void,
    stub_out: *mut u64,
) -> i32 {
    if !original.is_null() {
        unsafe { *original = ptr::null_mut() };
    }
    if !stub_out.is_null() {
        unsafe { *stub_out = 0 };
    }

    let caller_path = match unsafe { c_string_to_str(caller_path) } {
        Some(value) if !value.is_empty() => value,
        _ => return SrxHookErrno::InvalidArg.as_i32(),
    };
    let symbol = match unsafe { c_string_to_str(symbol) } {
        Some(value) if !value.is_empty() => value,
        _ => return SrxHookErrno::InvalidArg.as_i32(),
    };
    if replacement.is_null() {
        return SrxHookErrno::InvalidArg.as_i32();
    }

    let init_status = srx_hook::init(HookMode::Manual, false);
    if init_status != SrxHookErrno::Ok {
        return init_status.as_i32();
    }

    let capture = Box::into_raw(Box::new(OriginalCapture {
        original: ptr::null_mut(),
        error_status: SrxHookErrno::Ok.as_i32(),
    }));
    let stub = srx_hook::hook_single(
        caller_path,
        None,
        symbol,
        replacement,
        Some(capture_original),
        capture.cast(),
    );
    let Some(stub) = stub else {
        drop(unsafe { Box::from_raw(capture) });
        return SrxHookErrno::InvalidArg.as_i32();
    };
    insert_capture(stub, capture);

    let (refresh_status, _errors) = srx_hook::refresh();
    let (resolved, error_status) = match capture_for(stub) {
        Some(value) => {
            let capture = unsafe { &*value };
            (capture.original, capture.error_status)
        }
        None => (ptr::null_mut(), SrxHookErrno::Ok.as_i32()),
    };

    if refresh_status != SrxHookErrno::Ok {
        if srx_hook::unhook(stub) == SrxHookErrno::Ok {
            unsafe { release_capture(stub) };
        }
        return refresh_status.as_i32();
    }
    if resolved.is_null() {
        if srx_hook::unhook(stub) == SrxHookErrno::Ok {
            unsafe { release_capture(stub) };
        }
        if error_status != SrxHookErrno::Ok.as_i32() {
            return error_status;
        }
        return SrxHookErrno::NoSym.as_i32();
    }

    if !original.is_null() {
        unsafe { *original = resolved };
    }
    if !stub_out.is_null() {
        unsafe { *stub_out = stub };
    }
    SrxHookErrno::Ok.as_i32()
}

#[unsafe(no_mangle)]
pub extern "C" fn srx_hook_ffi_unhook(stub: u64) -> i32 {
    let status = srx_hook::unhook(stub);
    if status == SrxHookErrno::Ok {
        unsafe { release_capture(stub) };
    }
    status.as_i32()
}
