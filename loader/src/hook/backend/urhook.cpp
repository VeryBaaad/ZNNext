/*
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

#include <map>
#include <mutex>
#include <string>

#include "hook/backend.h"
#include "log.h"

#if defined(__aarch64__)
#include <ur/capi.h>
#endif

namespace znn::hook::backend {

#if defined(__aarch64__)

namespace {

struct InlineRecord {
  ur_inline_hook_t* handle = nullptr;
};

struct PltRecord {
  ur_plthook_t* handle = nullptr;
  void* original = nullptr;
};

std::mutex g_inline_mutex;
std::map<uintptr_t, InlineRecord> g_inline_records;

std::mutex g_plt_mutex;
std::map<std::string, PltRecord> g_plt_records;

std::string pltRecordKey(const std::string& caller_path, const char* symbol) {
  std::string key = caller_path;
  key += '\x01';
  key += symbol;
  return key;
}

bool openPltHandle(const std::string& caller_path, uintptr_t caller_base,
                   ur_plthook_t** out) {
  *out = nullptr;
  if (caller_base != 0 &&
      ur_plthook_create_from_base(caller_base, out) == UR_STATUS_OK && *out &&
      ur_plthook_is_valid(*out)) {
    return true;
  }
  if (*out) {
    ur_plthook_destroy(*out);
    *out = nullptr;
  }
  if (!caller_path.empty() &&
      ur_plthook_create_from_path(caller_path.c_str(), out) == UR_STATUS_OK &&
      *out && ur_plthook_is_valid(*out)) {
    return true;
  }
  if (*out) {
    ur_plthook_destroy(*out);
    *out = nullptr;
  }
  LOGE("pltHook: cannot parse %s (base %p)", caller_path.c_str(),
       reinterpret_cast<void*>(caller_base));
  return false;
}

}  // namespace

bool urhookHook(void* target, void* replacement, void** original) {
  if (!target || !replacement) return false;

  const uintptr_t key = reinterpret_cast<uintptr_t>(target);
  std::lock_guard<std::mutex> lk(g_inline_mutex);
  if (g_inline_records.find(key) != g_inline_records.end()) {
    LOGE("inlineHook %p failed: already hooked", target);
    return false;
  }

  ur_inline_hook_t* handle = nullptr;
  if (ur_inline_hook_create(key, replacement, 1, &handle) != UR_STATUS_OK ||
      !handle) {
    LOGE("inlineHook %p failed", target);
    return false;
  }

  void* trampoline = ur_inline_hook_get_trampoline(handle);
  if (!trampoline) {
    ur_inline_hook_destroy(handle);
    LOGE("inlineHook %p failed: no trampoline", target);
    return false;
  }

  g_inline_records.emplace(key, InlineRecord{handle});
  if (original) *original = trampoline;
  return true;
}

bool urhookUnhook(void* target) {
  const uintptr_t key = reinterpret_cast<uintptr_t>(target);
  std::lock_guard<std::mutex> lk(g_inline_mutex);
  const auto it = g_inline_records.find(key);
  if (it == g_inline_records.end()) {
    LOGE("inlineUnhook %p failed: not hooked", target);
    return false;
  }
  ur_inline_hook_destroy(it->second.handle);
  g_inline_records.erase(it);
  return true;
}

bool urhookPltHook(const std::string& caller_path, uintptr_t caller_base,
                   const char* symbol, void* replacement, void** original) {
  if (caller_path.empty() || !symbol || !replacement) return false;

  const std::string key = pltRecordKey(caller_path, symbol);
  std::lock_guard<std::mutex> lk(g_plt_mutex);

  const auto it = g_plt_records.find(key);
  if (it != g_plt_records.end()) {
    PltRecord& record = it->second;
    if (replacement != record.original) {
      LOGE("pltHook %s: %s is already hooked, unhook first", symbol,
           caller_path.c_str());
      return false;
    }
    if (ur_plthook_unhook_symbol(record.handle, symbol) != UR_STATUS_OK) {
      LOGE("pltHook %s: cannot restore %s", symbol, caller_path.c_str());
      return false;
    }
    ur_plthook_destroy(record.handle);
    void* const previous = record.original;
    g_plt_records.erase(it);
    if (original) *original = previous;
    return true;
  }

  ur_plthook_t* handle = nullptr;
  if (!openPltHandle(caller_path, caller_base, &handle)) return false;

  void* backup = nullptr;
  if (ur_plthook_hook_symbol(handle, symbol, replacement, &backup) !=
      UR_STATUS_OK) {
    ur_plthook_destroy(handle);
    LOGE("pltHook %s: no matching relocation in %s", symbol,
         caller_path.c_str());
    return false;
  }

  g_plt_records.emplace(key, PltRecord{handle, backup});
  if (original) *original = backup;
  return true;
}

#else

bool urhookHook(void* target, void* replacement, void** original) {
  (void)target;
  (void)replacement;
  (void)original;
  return false;
}

bool urhookUnhook(void* target) {
  (void)target;
  return false;
}

bool urhookPltHook(const std::string& caller_path, uintptr_t caller_base,
                   const char* symbol, void* replacement, void** original) {
  (void)caller_path;
  (void)caller_base;
  (void)symbol;
  (void)replacement;
  (void)original;
  return false;
}

#endif

}  // namespace znn::hook::backend
