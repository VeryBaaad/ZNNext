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

#include <cstdint>
#include <map>
#include <mutex>
#include <set>

#include "hook/backend.h"
#include "log.h"

#if defined(__aarch64__) || defined(__x86_64__)
#include "srx_inline_hook_ffi.h"
#endif

namespace znn::hook::backend {

#if defined(__aarch64__) || defined(__x86_64__)

namespace {

std::once_flag g_init_once;
int32_t g_init_result = -1;
std::once_flag g_ready_log_once;
std::mutex g_mutex;
std::map<uintptr_t, void*> g_stubs;
std::set<uintptr_t> g_unhooking;

bool ensureReady() {
  std::call_once(g_init_once,
                 [] { g_init_result = srx_inline_hook_ffi_init(); });
  if (g_init_result != 0) {
    LOGE("srx_inline_hook init failed (%d)", g_init_result);
    return false;
  }
  return true;
}

}  // namespace

bool srxinlinehookInit() {
  if (!ensureReady()) return false;
  std::call_once(g_ready_log_once,
                 [] { LOGI("srx_inline_hook engine ready (mode: unique)"); });
  return true;
}

bool srxinlinehookHook(void* target, void* replacement, void** original) {
  if (!target || !replacement) return false;
  if (!srxinlinehookInit()) return false;

  const uintptr_t key = reinterpret_cast<uintptr_t>(target);
  {
    std::lock_guard<std::mutex> lk(g_mutex);
    if (g_stubs.find(key) != g_stubs.end()) {
      LOGE("inlineHook %p already hooked by srx_inline_hook", target);
      return false;
    }
  }

  void* stub = nullptr;
  const int32_t status =
      srx_inline_hook_ffi_hook(target, replacement, original, &stub);
  if (status != 0 || !stub) {
    LOGE("inlineHook %p failed: srx_inline_hook errno=%d", target, status);
    return false;
  }

  {
    std::lock_guard<std::mutex> lk(g_mutex);
    g_stubs.emplace(key, stub);
  }
  return true;
}

bool srxinlinehookUnhook(void* target) {
  if (!target) return false;
  if (!srxinlinehookInit()) return false;

  const uintptr_t key = reinterpret_cast<uintptr_t>(target);
  void* stub = nullptr;
  {
    std::lock_guard<std::mutex> lk(g_mutex);
    const auto it = g_stubs.find(key);
    if (it == g_stubs.end()) {
      LOGE("inlineUnhook %p: srx_inline_hook has no such hook", target);
      return false;
    }
    if (!g_unhooking.insert(key).second) {
      LOGE("inlineUnhook %p is already in progress", target);
      return false;
    }
    stub = it->second;
  }

  const int32_t status = srx_inline_hook_ffi_unhook(stub);

  {
    std::lock_guard<std::mutex> lk(g_mutex);
    g_unhooking.erase(key);
    if (status == 0) g_stubs.erase(key);
  }

  if (status != 0) {
    LOGE("inlineUnhook %p failed: srx_inline_hook errno=%d", target, status);
    return false;
  }
  return true;
}

#else

bool srxinlinehookInit() { return false; }

bool srxinlinehookHook(void* target, void* replacement, void** original) {
  (void)target;
  (void)replacement;
  (void)original;
  return false;
}

bool srxinlinehookUnhook(void* target) {
  (void)target;
  return false;
}

#endif

}  // namespace znn::hook::backend
