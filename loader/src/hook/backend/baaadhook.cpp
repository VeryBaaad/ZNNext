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

#include <bhinline.h>
#include <bhplt.h>

#include <map>
#include <mutex>
#include <string>

#include "hook/backend.h"
#include "log.h"

namespace znn::hook::backend {
namespace {

struct PltRecord {
  void* replacement = nullptr;
  void* original = nullptr;
};

std::mutex g_plt_mutex;
std::map<std::string, PltRecord> g_plt_records;

std::string pltRecordKey(const std::string& caller_path, const char* symbol) {
  std::string key = caller_path;
  key += '\x01';
  key += symbol;
  return key;
}

}  // namespace

bool baaadhookHook(void* target, void* replacement, void** original) {
  const int status = bh_inline_hook(target, replacement, original);
  if (status != BH_INLINE_OK) {
    LOGE("inlineHook %p failed: %s", target, bh_inline_strerror(status));
    return false;
  }
  return true;
}

bool baaadhookUnhook(void* target) {
  const int status = bh_inline_unhook(target);
  if (status != BH_INLINE_OK) {
    LOGE("inlineUnhook %p failed: %s", target, bh_inline_strerror(status));
    return false;
  }
  return true;
}

bool baaadhookPltHook(const std::string& caller_path, const char* symbol,
                      void* replacement, void** original) {
  if (caller_path.empty() || !symbol || !replacement) return false;

  const std::string key = pltRecordKey(caller_path, symbol);
  std::lock_guard<std::mutex> lk(g_plt_mutex);

  if (const auto it = g_plt_records.find(key); it != g_plt_records.end()) {
    PltRecord& record = it->second;
    if (replacement != record.original) {
      LOGE("pltHook %s: %s is already hooked, unhook first", symbol,
           caller_path.c_str());
      return false;
    }
    if (bh_plt_unhook(caller_path.c_str(), symbol, record.replacement) <= 0) {
      LOGE("pltHook %s: cannot restore slot in %s", symbol,
           caller_path.c_str());
      return false;
    }
    void* const previous = record.original;
    g_plt_records.erase(it);
    if (original) *original = previous;
    return true;
  }

  void* resolved = nullptr;
  const int status =
      bh_plt_hook(caller_path.c_str(), symbol, replacement, &resolved);
  if (status < 0) {
    LOGE("pltHook %s in %s failed: %s", symbol, caller_path.c_str(),
         bh_plt_strerror(status));
    return false;
  }
  if (!resolved) {
    LOGE("pltHook %s in %s failed: cannot resolve the original symbol", symbol,
         caller_path.c_str());
    bh_plt_unhook(caller_path.c_str(), symbol, replacement);
    return false;
  }

  g_plt_records.emplace(key, PltRecord{replacement, resolved});
  if (original) *original = resolved;
  return true;
}

}  // namespace znn::hook::backend
