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

#include <stdint.h>

#include "hook/backend.h"
#include "log.h"

#if defined(__aarch64__) || defined(__x86_64__)

#include <map>
#include <mutex>

#include "srx_hook_ffi.h"

namespace znn::hook::backend {
namespace {

struct PltRecord {
  uint64_t stub = 0;
  void* replacement = nullptr;
  void* original = nullptr;
};

std::mutex g_mutex;
bool g_ready = false;
std::map<std::string, PltRecord> g_records;

std::string recordKey(const std::string& caller_path, const char* symbol) {
  std::string key = caller_path;
  key += '\x01';
  key += symbol;
  return key;
}

bool ensureReadyLocked() {
  if (g_ready) return true;
  const int32_t status = srx_hook_ffi_init();
  if (status != 0) {
    LOGE("srx_hook: init failed (%d)", status);
    return false;
  }
  g_ready = true;
  LOGI("srx_hook engine ready (mode: manual)");
  return true;
}

}  // namespace

bool srxhookInit() {
  std::lock_guard<std::mutex> lk(g_mutex);
  return ensureReadyLocked();
}

bool srxhookPltHook(const std::string& caller_path, uintptr_t caller_base,
                    const char* symbol, void* replacement, void** original) {
  (void)caller_base;
  if (caller_path.empty() || !symbol || !replacement) return false;

  const std::string key = recordKey(caller_path, symbol);
  std::lock_guard<std::mutex> lk(g_mutex);

  const auto it = g_records.find(key);
  if (it != g_records.end()) {
    PltRecord& record = it->second;
    if (replacement != record.original) {
      LOGE("pltHook %s: %s is already hooked, unhook first", symbol,
           caller_path.c_str());
      return false;
    }
    const int32_t status = srx_hook_ffi_unhook(record.stub);
    if (status != 0) {
      LOGE("pltHook %s: cannot restore %s (%d)", symbol, caller_path.c_str(),
           status);
      return false;
    }
    void* const previous = record.original;
    g_records.erase(it);
    if (original) *original = previous;
    return true;
  }

  if (!ensureReadyLocked()) return false;

  uint64_t stub = 0;
  void* resolved = nullptr;
  const int32_t status = srx_hook_ffi_hook(caller_path.c_str(), symbol,
                                           replacement, &resolved, &stub);
  if (status != 0) {
    LOGE("pltHook %s: srx_hook failed for %s (%d)", symbol, caller_path.c_str(),
         status);
    return false;
  }

  g_records.emplace(key, PltRecord{stub, replacement, resolved});
  if (original) *original = resolved;
  return true;
}

}  // namespace znn::hook::backend

#else

namespace znn::hook::backend {

bool srxhookInit() { return false; }

bool srxhookPltHook(const std::string& caller_path, uintptr_t caller_base,
                    const char* symbol, void* replacement, void** original) {
  (void)caller_path;
  (void)caller_base;
  (void)symbol;
  (void)replacement;
  (void)original;
  return false;
}

}  // namespace znn::hook::backend

#endif
