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
#include <memory>
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

struct Slot {
  void* value = nullptr;
};

struct Pending {
  void* replacement = nullptr;
  bool restoring = false;
  std::unique_ptr<Slot> slot;
};

std::mutex g_mutex;
std::map<std::string, Record> g_records;
std::map<std::string, Pending> g_pending;

std::string recordKey(dev_t dev, ino_t inode, const char* symbol) {
  std::string key = std::to_string(static_cast<unsigned long long>(dev));
  key += ':';
  key += std::to_string(static_cast<unsigned long long>(inode));
  key += '\x01';
  key += symbol;
  return key;
}

void reconcileLocked() {
  for (auto it = g_pending.begin(); it != g_pending.end();) {
    if (it->second.slot->value == nullptr) {
      ++it;
      continue;
    }
    if (it->second.restoring) {
      auto record = g_records.find(it->first);
      if (record != g_records.end() &&
          record->second.replacement == it->second.replacement) {
        g_records.erase(record);
      }
    } else {
      g_records[it->first] =
          Record{it->second.replacement, it->second.slot->value};
    }
    it = g_pending.erase(it);
  }
}

bool finishPending(std::map<std::string, Pending>::iterator pending,
                   bool committed) {
  void* const original = pending->second.slot->value;
  if (!original) {
    if (!committed) {
      LOGE("pltHook: CommitHook failed");
    } else {
      LOGE("pltHook: symbol not found in PLT");
    }
    return false;
  }
  if (!committed) {
    LOGW("pltHook: CommitHook reported failures for another registration");
  }
  if (pending->second.restoring) {
    g_records.erase(pending->first);
  } else {
    g_records[pending->first] = Record{pending->second.replacement, original};
  }
  g_pending.erase(pending);
  return true;
}

}  // namespace

bool lspltHook(dev_t dev, ino_t inode, const char* symbol, void* replacement,
               void** original) {
  if (!symbol) return false;

  std::lock_guard<std::mutex> lk(g_mutex);
  reconcileLocked();

  const std::string key = recordKey(dev, inode, symbol);
  auto record = g_records.find(key);
  if (record != g_records.end()) {
    void* const previous = record->second.original;
    if (replacement != previous) {
      LOGE("pltHook %s: already hooked, unhook first", symbol);
      return false;
    }

    auto pending = g_pending.find(key);
    if (pending == g_pending.end()) {
      auto slot = std::make_unique<Slot>();
      Pending state;
      state.replacement = record->second.replacement;
      state.restoring = true;
      state.slot = std::move(slot);
      pending = g_pending.emplace(key, std::move(state)).first;
      if (!lsplt::RegisterHook(dev, inode, symbol, previous,
                               &pending->second.slot->value)) {
        g_pending.erase(pending);
        LOGE("pltHook %s: RegisterHook failed", symbol);
        return false;
      }
    }

    const bool committed = lsplt::CommitHook();
    if (!finishPending(pending, committed)) return false;
    if (original) *original = previous;
    return true;
  }

  auto pending = g_pending.find(key);
  if (pending != g_pending.end()) {
    if (pending->second.restoring ||
        pending->second.replacement != replacement) {
      LOGE("pltHook %s: another installation is pending", symbol);
      return false;
    }
    const bool committed = lsplt::CommitHook();
    if (!finishPending(pending, committed)) return false;
    if (original) *original = g_records[key].original;
    return true;
  }

  auto slot = std::make_unique<Slot>();
  Pending state;
  state.replacement = replacement;
  state.restoring = false;
  state.slot = std::move(slot);
  pending = g_pending.emplace(key, std::move(state)).first;
  if (!lsplt::RegisterHook(dev, inode, symbol, replacement,
                           &pending->second.slot->value)) {
    g_pending.erase(pending);
    LOGE("pltHook %s: RegisterHook failed", symbol);
    return false;
  }

  const bool committed = lsplt::CommitHook();
  if (!finishPending(pending, committed)) return false;
  if (original) *original = g_records[key].original;
  return true;
}

}  // namespace znn::hook::backend
