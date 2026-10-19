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

#include <lsplt.hpp>
#include <map>
#include <mutex>
#include <string>

#include "hook/backend.h"
#include "log.h"

namespace znn::hook::backend {
namespace {

struct Record {
  void* replacement = nullptr;
  void* original = nullptr;
};

std::mutex g_mutex;
std::map<std::string, Record> g_records;

std::string recordKey(dev_t dev, ino_t inode, const char* symbol) {
  std::string key = std::to_string(static_cast<unsigned long long>(dev));
  key += ':';
  key += std::to_string(static_cast<unsigned long long>(inode));
  key += '\x01';
  key += symbol;
  return key;
}

}  // namespace

bool lspltHook(dev_t dev, ino_t inode, const char* symbol, void* replacement,
               void** original) {
  const std::string key = recordKey(dev, inode, symbol);

  void* previous = nullptr;
  {
    std::lock_guard<std::mutex> lk(g_mutex);
    if (auto it = g_records.find(key); it != g_records.end())
      previous = it->second.original;
  }

  void* backup = nullptr;
  if (previous) {
    if (replacement != previous) {
      LOGE("pltHook %s: already hooked, unhook first", symbol);
      return false;
    }

    lsplt::RegisterHook(dev, inode, symbol, previous, &backup);
    const bool committed = lsplt::CommitHook();
    if (!backup) {
      if (!committed) {
        LOGE("pltHook %s: CommitHook failed", symbol);
      } else {
        LOGE("pltHook %s: symbol not found in PLT", symbol);
      }
      return false;
    }
    if (!committed) {
      LOGW("pltHook %s: the commit failed after the hook was removed", symbol);
    }
    {
      std::lock_guard<std::mutex> lk(g_mutex);
      g_records.erase(key);
    }
    if (original) *original = previous;
    return true;
  }

  lsplt::RegisterHook(dev, inode, symbol, replacement, &backup);
  const bool committed = lsplt::CommitHook();
  if (!backup) {
    if (!committed) {
      LOGE("pltHook %s: CommitHook failed", symbol);
    } else {
      LOGE("pltHook %s: symbol not found in PLT", symbol);
    }
    return false;
  }
  if (!committed) {
    LOGW("pltHook %s: the commit failed after the hook was installed", symbol);
  }
  {
    std::lock_guard<std::mutex> lk(g_mutex);
    g_records[key] = Record{replacement, backup};
  }
  if (original) *original = backup;
  return true;
}

}  // namespace znn::hook::backend
