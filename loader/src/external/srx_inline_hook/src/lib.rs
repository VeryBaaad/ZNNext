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

use std::ffi::c_void;
use std::ptr;

use srx_inline_hook::{Errno, HookMode, HookStub, hook_with_orig, init, unhook};

struct StubHandle {
    stub: HookStub,
}

#[unsafe(no_mangle)]
pub extern "C" fn srx_inline_hook_ffi_init() -> i32 {
    match init(HookMode::Unique, false) {
        Ok(()) => 0,
        Err(error) => error as i32,
    }
}

/// Safety: export
#[unsafe(no_mangle)]
pub unsafe extern "C" fn srx_inline_hook_ffi_hook(
    target: *mut c_void,
    replacement: *mut c_void,
    original: *mut *mut c_void,
    stub: *mut *mut c_void,
) -> i32 {
    if !original.is_null() {
        unsafe { *original = ptr::null_mut() };
    }
    if !stub.is_null() {
        unsafe { *stub = ptr::null_mut() };
    }
    if target.is_null() || replacement.is_null() || stub.is_null() {
        return Errno::InvalidArg as i32;
    }

    let mut previous: *const c_void = ptr::null();
    let result = unsafe { hook_with_orig(target, replacement, &mut previous) };
    let installed = match result {
        Ok(value) => value,
        Err(error) => return error as i32,
    };

    if !original.is_null() {
        unsafe { *original = previous.cast_mut() };
    }
    let handle = Box::into_raw(Box::new(StubHandle { stub: installed }));
    unsafe { *stub = handle.cast() };
    0
}

/// Safety: export
#[unsafe(no_mangle)]
pub unsafe extern "C" fn srx_inline_hook_ffi_unhook(stub: *mut c_void) -> i32 {
    if stub.is_null() {
        return Errno::InvalidArg as i32;
    }

    let handle = unsafe { Box::from_raw(stub.cast::<StubHandle>()) };
    match unsafe { unhook(handle.stub) } {
        Ok(()) => 0,
        Err(error) => {
            let _ = Box::into_raw(handle);
            error as i32
        }
    }
}
