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

#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include <map>
#include <mutex>

#include "hook/backend.h"
#include "log.h"

#if defined(__aarch64__) || defined(__arm__) || defined(__x86_64__)
#include <hookzz.h>
#endif

namespace znn::hook::backend {

#if defined(__aarch64__) || defined(__arm__) || defined(__x86_64__)

namespace {

#if defined(__aarch64__)
// ldr x17, #8; br x17; <address>
constexpr size_t kPatchBytes = 16;
#elif defined(__arm__)
// ldr pc, [pc, #-4]; <address>
constexpr size_t kPatchBytes = 8;
#else
// jmp qword ptr [rip]; <address>
constexpr size_t kPatchBytes = 14;
#endif

struct HookRecord {
  uint8_t backup[kPatchBytes];
};

std::mutex g_mutex;
std::map<uintptr_t, HookRecord> g_records;

bool makePatchWritable(void* target) {
  const long page_size = sysconf(_SC_PAGESIZE);
  if (page_size <= 0) return false;

  const uintptr_t align_mask = static_cast<uintptr_t>(page_size) - 1;
  const uintptr_t begin = reinterpret_cast<uintptr_t>(target) & ~align_mask;
  const uintptr_t end =
      (reinterpret_cast<uintptr_t>(target) + kPatchBytes + align_mask) &
      ~align_mask;
  return mprotect(reinterpret_cast<void*>(begin), end - begin,
                  PROT_READ | PROT_WRITE | PROT_EXEC) == 0;
}

void flushPatch(void* target) {
  __builtin___clear_cache(reinterpret_cast<char*>(target),
                          reinterpret_cast<char*>(target) + kPatchBytes);
}

}  // namespace

bool hookzzHook(void* target, void* replacement, void** original) {
  if (!target || !replacement) return false;

  std::lock_guard<std::mutex> lk(g_mutex);
  const uintptr_t key = reinterpret_cast<uintptr_t>(target);
  if (g_records.find(key) != g_records.end()) {
    LOGE("inlineHook %p failed: already hooked", target);
    return false;
  }

  HookRecord record{};
  memcpy(record.backup, target, kPatchBytes);

  void* relocated = nullptr;
  if (ZzReplace(target, replacement, &relocated) != RS_SUCCESS) {
    LOGE("inlineHook %p failed", target);
    return false;
  }

  g_records.emplace(key, record);
  if (original) *original = relocated;
  return true;
}

bool hookzzUnhook(void* target) {
  std::lock_guard<std::mutex> lk(g_mutex);
  const uintptr_t key = reinterpret_cast<uintptr_t>(target);

  const auto it = g_records.find(key);
  if (it == g_records.end()) {
    LOGE("inlineUnhook %p failed: not hooked", target);
    return false;
  }
  if (!makePatchWritable(target)) {
    LOGE("inlineUnhook %p failed: the target is not writable", target);
    return false;
  }

  memcpy(target, it->second.backup, kPatchBytes);
  flushPatch(target);
  g_records.erase(it);
  return true;
}

#else

bool hookzzHook(void* target, void* replacement, void** original) {
  (void)target;
  (void)replacement;
  (void)original;
  return false;
}

bool hookzzUnhook(void* target) {
  (void)target;
  return false;
}

#endif

}  // namespace znn::hook::backend
