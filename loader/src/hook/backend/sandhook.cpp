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

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include <cstdint>
#include <map>
#include <mutex>

#include "hook/backend.h"
#include "log.h"
#include "utils/elf_util.h"

#if defined(__aarch64__) || defined(__arm__)
#include <sandhook_native.h>

#undef BreakCallback
#undef EXPORT
#undef REG
#endif

namespace znn::hook::backend {

#if defined(__aarch64__) || defined(__arm__)

namespace {

constexpr size_t kPatchBytes = 16;

struct HookRecord {
  uint8_t backup[kPatchBytes];
  int prot;
};

std::mutex g_mutex;
std::map<uintptr_t, HookRecord> g_records;

#if defined(__arm__) && (!defined(__ARM_ARCH) || __ARM_ARCH < 7)

bool hasThumb2() {
  FILE* file = fopen("/proc/cpuinfo", "r");
  if (!file) return false;

  char line[256];
  bool supported = false;
  while (fgets(line, sizeof(line), file)) {
    if (strncmp(line, "Features", 8) == 0 && strstr(line, "thumb2")) {
      supported = true;
      break;
    }
    if (strncmp(line, "CPU architecture", 16) != 0) continue;
    const char* value = strchr(line, ':');
    if (!value) continue;
    char* end = nullptr;
    const long architecture = strtol(value + 1, &end, 10);
    if (end == value + 1) continue;
    supported = architecture >= 7;
    break;
  }
  fclose(file);
  return supported;
}

#endif

uintptr_t codeAddress(void* target) {
  uintptr_t address = reinterpret_cast<uintptr_t>(target);
#if defined(__arm__)
  address &= ~static_cast<uintptr_t>(1);
#endif
  return address;
}

int mappingProtection(void* target) {
  const uintptr_t address = codeAddress(target);
  for (const auto& mapping : znn::parseMaps("self")) {
    if (address < mapping.start || address >= mapping.end) continue;
    int prot = 0;
    if (mapping.perms & PROT_READ) prot |= PROT_READ;
    if (mapping.perms & PROT_WRITE) prot |= PROT_WRITE;
    if (mapping.perms & PROT_EXEC) prot |= PROT_EXEC;
    return prot;
  }
  return PROT_READ | PROT_EXEC;
}

bool setPageProtection(void* target, int prot) {
  const long page_size = sysconf(_SC_PAGESIZE);
  if (page_size <= 0) return false;

  const uintptr_t address = codeAddress(target);
  const uintptr_t mask = static_cast<uintptr_t>(page_size) - 1;
  const uintptr_t begin = address & ~mask;
  const uintptr_t end = (address + kPatchBytes + mask) & ~mask;
  return mprotect(reinterpret_cast<void*>(begin),
                  static_cast<size_t>(end - begin), prot) == 0;
}

}  // namespace

bool sandhookInit() {
#if defined(__arm__) && (!defined(__ARM_ARCH) || __ARM_ARCH < 7)
  static std::once_flag once;
  static bool supported = true;
  std::call_once(once, [] { supported = hasThumb2(); });
  if (!supported) {
    LOGE("sandhook: Thumb-2 is unavailable, so ARMv7 or newer is required");
    return false;
  }
#endif
  return true;
}

bool sandhookHook(void* target, void* replacement, void** original) {
  if (!target || !replacement) return false;

  void* code = reinterpret_cast<void*>(codeAddress(target));
  const uintptr_t key = reinterpret_cast<uintptr_t>(code);
  std::lock_guard<std::mutex> lk(g_mutex);
  if (g_records.find(key) != g_records.end()) {
    LOGE("inlineHook %p failed: already hooked", target);
    return false;
  }

  HookRecord record{};
  memcpy(record.backup, code, kPatchBytes);
  record.prot = mappingProtection(code);

  void* trampoline = SandInlineHook(target, replacement);
  if (!trampoline) {
    LOGE("inlineHook %p failed: SandHook rejected the target", target);
    return false;
  }

  if (!setPageProtection(target, record.prot)) {
    LOGW("inlineHook %p: cannot restore the code page protection", target);
  }

  g_records.emplace(key, record);
  if (original) *original = trampoline;
  return true;
}

bool sandhookUnhook(void* target) {
  if (!target) return false;

  void* code = reinterpret_cast<void*>(codeAddress(target));
  const uintptr_t key = reinterpret_cast<uintptr_t>(code);
  std::lock_guard<std::mutex> lk(g_mutex);
  const auto it = g_records.find(key);
  if (it == g_records.end()) {
    LOGE("inlineUnhook %p failed: not hooked", target);
    return false;
  }

  const int prot = it->second.prot;
  if (!setPageProtection(code, prot | PROT_WRITE | PROT_EXEC)) {
    LOGE("inlineUnhook %p failed: the target is not writable", target);
    return false;
  }

  memcpy(code, it->second.backup, kPatchBytes);
  __builtin___clear_cache(reinterpret_cast<char*>(code),
                          reinterpret_cast<char*>(code) + kPatchBytes);

  if (!setPageProtection(target, prot)) {
    LOGW("inlineUnhook %p: cannot restore the code page protection", target);
  }
  g_records.erase(it);
  return true;
}

#else

bool sandhookInit() { return false; }

bool sandhookHook(void* target, void* replacement, void** original) {
  (void)target;
  (void)replacement;
  (void)original;
  return false;
}

bool sandhookUnhook(void* target) {
  (void)target;
  return false;
}

#endif

}  // namespace znn::hook::backend
