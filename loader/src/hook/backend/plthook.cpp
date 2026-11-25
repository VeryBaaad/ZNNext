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

#include <sys/mman.h>
#include <unistd.h>

#include <cerrno>
#include <cstring>
#include <map>
#include <mutex>
#include <string>
#include <vector>

#include "hook/backend.h"
#include "log.h"

#ifndef __riscv
#include <plthook.h>
#endif

namespace znn::hook::backend {

#ifndef __riscv

namespace {

struct Slot {
  void** addr;
  void* previous;
  int prot;
};

struct Record {
  void* replacement;
  void* original;
  std::vector<Slot> slots;
};

std::mutex g_mutex;
std::map<std::string, Record> g_records;

std::string recordKey(const PltTarget& target, const char* symbol) {
  std::string key = std::to_string(target.base);
  key += '\x01';
  key += target.maps_path;
  key += '\x01';
  key += symbol;
  return key;
}

size_t pageSize() {
  static const size_t size = static_cast<size_t>(sysconf(_SC_PAGESIZE));
  return size;
}

bool writeSlot(const Slot& slot, void* value) {
  auto* page = reinterpret_cast<void*>(reinterpret_cast<uintptr_t>(slot.addr) &
                                       ~(pageSize() - 1));
  const bool restore = !(slot.prot & PROT_WRITE);
  if (restore && mprotect(page, pageSize(), slot.prot | PROT_WRITE) != 0) {
    LOGE("pltHook: mprotect %p to write failed: %s", page, strerror(errno));
    return false;
  }
  *slot.addr = value;
  if (restore && mprotect(page, pageSize(), slot.prot) != 0) {
    LOGE("pltHook: mprotect %p to restore failed: %s", page, strerror(errno));
    return false;
  }
  return true;
}

bool symbolMatches(const char* name, const char* symbol, size_t length) {
  return strncmp(name, symbol, length) == 0 &&
         (name[length] == '\0' || name[length] == '@');
}

bool collectSlots(plthook_t* handle, const char* symbol,
                  std::vector<Slot>* slots) {
  const size_t length = strlen(symbol);
  unsigned int pos = 0;
  const char* name = nullptr;
  void** addr = nullptr;
  int prot = 0;
  int rv;
  while ((rv = plthook_enum_with_prot(handle, &pos, &name, &addr, &prot)) ==
         0) {
    if (!symbolMatches(name, symbol, length)) continue;
    if (!addr) continue;
    if (prot == 0) {
      LOGE("pltHook %s: no memory protection for slot %p", symbol, addr);
      return false;
    }
    slots->push_back(Slot{addr, *addr, prot});
  }
  if (rv != EOF) {
    LOGE("pltHook %s: plthook_enum failed: %s", symbol, plthook_error());
    return false;
  }
  return true;
}

bool openPlthook(const std::string& path, uintptr_t base, plthook_t** out) {
  if (base != 0 && plthook_open_by_address(
                       out, reinterpret_cast<void*>(base)) == PLTHOOK_SUCCESS) {
    return true;
  }
  if (!path.empty() && plthook_open(out, path.c_str()) == PLTHOOK_SUCCESS) {
    return true;
  }
  LOGE("pltHook: cannot open %s (base %p): %s", path.c_str(),
       reinterpret_cast<void*>(base), plthook_error());
  return false;
}

}  // namespace

bool plthookHook(const PltTarget& target, const char* symbol, void* replacement,
                 void** original) {
  const std::string& caller_path = target.maps_path;
  if (caller_path.empty() || !symbol || !replacement) return false;

  const std::string key = recordKey(target, symbol);
  std::lock_guard<std::mutex> lk(g_mutex);

  if (auto it = g_records.find(key); it != g_records.end()) {
    Record& record = it->second;
    if (replacement != record.original) {
      LOGE("pltHook %s: %s is already hooked, unhook first", symbol,
           caller_path.c_str());
      return false;
    }
    size_t restored = 0;
    for (; restored < record.slots.size(); ++restored) {
      if (!writeSlot(record.slots[restored], record.slots[restored].previous)) {
        LOGE("pltHook %s: cannot restore slot in %s", symbol,
             caller_path.c_str());
        for (size_t i = 0; i <= restored; ++i)
          writeSlot(record.slots[i], record.replacement);
        return false;
      }
    }
    void* const previous = record.original;
    g_records.erase(it);
    if (original) *original = previous;
    return true;
  }

  plthook_t* handle = nullptr;
  if (!openPlthook(caller_path, target.base, &handle)) return false;

  std::vector<Slot> slots;
  const bool collected = collectSlots(handle, symbol, &slots);
  plthook_close(handle);
  if (!collected) return false;
  if (slots.empty()) {
    LOGE("pltHook %s: no matching relocation in %s", symbol,
         caller_path.c_str());
    return false;
  }

  void* resolved = slots.front().previous;
  for (const Slot& slot : slots) {
    if (slot.previous != resolved) {
      LOGE("pltHook %s: slots in %s have different originals", symbol,
           caller_path.c_str());
      return false;
    }
  }
  if (original) *original = resolved;

  size_t hooked = 0;
  for (; hooked < slots.size(); ++hooked) {
    if (!writeSlot(slots[hooked], replacement)) {
      LOGE("pltHook %s: cannot hook slot in %s", symbol, caller_path.c_str());
      bool rollback_failed = false;
      for (size_t i = 0; i <= hooked; ++i) {
        if (!writeSlot(slots[i], slots[i].previous)) rollback_failed = true;
      }
      if (!rollback_failed) return false;

      LOGW("pltHook %s: rollback failed in %s, keeping the record", symbol,
           caller_path.c_str());
      Record record{replacement, resolved, std::move(slots)};
      g_records.emplace(key, std::move(record));
      return true;
    }
  }

  Record record{replacement, resolved, std::move(slots)};
  g_records.emplace(key, std::move(record));
  return true;
}

#else

bool plthookHook(const PltTarget& target, const char* symbol, void* replacement,
                 void** original) {
  (void)target;
  (void)symbol;
  (void)replacement;
  (void)original;
  return false;
}

#endif

}  // namespace znn::hook::backend
