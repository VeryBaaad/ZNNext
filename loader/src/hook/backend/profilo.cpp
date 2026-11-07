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

#include <exception>
#include <map>
#include <mutex>
#include <string>

#include "hook/backend.h"
#include "log.h"

#ifndef __riscv
#include <linker/sharedlibs.h>
#include <plthooks/plthooks.h>
#endif

namespace znn::hook::backend {

#ifndef __riscv

namespace {

struct Record {
  void* replacement = nullptr;
  void* original = nullptr;
};

std::mutex g_mutex;
std::map<std::string, Record> g_records;

std::string recordKey(const std::string& path, const char* symbol) {
  std::string key = path;
  key += '\x01';
  key += symbol;
  return key;
}

void restoreQuietly(const std::string& caller_path, const char* symbol,
                    void* replacement) {
  try {
    plt_hook_spec spec(symbol, replacement);
    unhook_single_lib(caller_path.c_str(), &spec, 1);
  } catch (...) {
  }
}

bool hookLocked(const std::string& caller_path, uintptr_t caller_base,
                const char* symbol, void* replacement, void** original) {
  const std::string key = recordKey(caller_path, symbol);

  if (const auto it = g_records.find(key); it != g_records.end()) {
    const Record& record = it->second;
    if (replacement != record.original) {
      LOGE("pltHook %s: %s is already hooked, unhook first", symbol,
           caller_path.c_str());
      return false;
    }
    plt_hook_spec spec(symbol, record.replacement);
    const int failures = unhook_single_lib(caller_path.c_str(), &spec, 1);
    if (failures != 0) {
      LOGE("pltHook %s: cannot restore slot in %s", symbol,
           caller_path.c_str());
      return false;
    }
    if (spec.hook_result == 0) {
      LOGW("pltHook %s: %s no longer has the hook, dropping the record", symbol,
           caller_path.c_str());
    }
    void* const previous = record.original;
    g_records.erase(it);
    if (original) *original = previous;
    return true;
  }

  auto lib = facebook::linker::sharedLib(caller_path.c_str());
  if (!lib.success) {
    LOGE("pltHook: no linker data for %s (base %p)", caller_path.c_str(),
         reinterpret_cast<void*>(caller_base));
    return false;
  }
  const uintptr_t load_bias = lib.data.getLoadBias();
  if (caller_base != 0 && load_bias != 0 && load_bias != caller_base) {
    LOGE("pltHook: %s is loaded at %p, not %p", caller_path.c_str(),
         reinterpret_cast<void*>(load_bias),
         reinterpret_cast<void*>(caller_base));
    return false;
  }
  auto* const elf_symbol = lib.data.find_symbol_by_name(symbol);
  if (!elf_symbol) {
    LOGE("pltHook %s: no such symbol in %s", symbol, caller_path.c_str());
    return false;
  }
  const auto slots = lib.data.get_plt_relocations(elf_symbol);
  if (slots.empty()) {
    LOGE("pltHook %s: no matching relocation in %s", symbol,
         caller_path.c_str());
    return false;
  }
  void* const previous = *slots.front();

  plt_hook_spec spec(symbol, replacement);
  const int failures = hook_single_lib(caller_path.c_str(), &spec, 1);
  if (failures != 0 || spec.hook_result == 0) {
    LOGE("pltHook %s in %s failed (%d errors, %d slots)", symbol,
         caller_path.c_str(), failures, spec.hook_result);
    restoreQuietly(caller_path, symbol, replacement);
    return false;
  }

  g_records.emplace(key, Record{replacement, previous});
  if (original) *original = previous;
  return true;
}

}  // namespace

bool profiloInit() {
  const int error = plthooks_initialize();
  if (error != 0) {
    LOGE("profilo: plthooks_initialize failed: %d", error);
    return false;
  }
  return true;
}

bool profiloHook(const std::string& caller_path, uintptr_t caller_base,
                 const char* symbol, void* replacement, void** original) {
  if (caller_path.empty() || !symbol || !replacement) return false;

  std::lock_guard<std::mutex> lk(g_mutex);
  try {
    return hookLocked(caller_path, caller_base, symbol, replacement, original);
  } catch (const std::exception& error) {
    LOGE("pltHook %s in %s failed: %s", symbol, caller_path.c_str(),
         error.what());
  } catch (...) {
    LOGE("pltHook %s in %s failed with an unknown error", symbol,
         caller_path.c_str());
  }
  restoreQuietly(caller_path, symbol, replacement);
  return false;
}

#else

bool profiloInit() { return false; }

bool profiloHook(const std::string& caller_path, uintptr_t caller_base,
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
