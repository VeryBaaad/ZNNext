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

#include <errno.h>
#include <fcntl.h>
#include <string.h>
#include <unistd.h>

#include <cstdint>
#include <map>
#include <mutex>

#include "hook/backend.h"
#include "log.h"

#if defined(__aarch64__)
#include "inline_hook.hpp"

namespace {

constexpr size_t kMaxPatchSize = 64;

}  // namespace

extern "C" uint8_t yz_patch_text_checked(uintptr_t addr, const void* expected,
                                         const void* replacement,
                                         unsigned int len) {
  const size_t size = len;
  if (addr == 0 || !expected || !replacement || size == 0 ||
      size > kMaxPatchSize || addr > UINTPTR_MAX - size)
    return 0;
  if (memcmp(reinterpret_cast<const void*>(addr), expected, size) != 0)
    return 0;

  const int fd = open("/proc/self/mem", O_RDWR | O_CLOEXEC);
  if (fd < 0) {
    LOGE("inlineHook: cannot open /proc/self/mem to patch %p",
         reinterpret_cast<const void*>(addr));
    return 0;
  }

  const auto* bytes = static_cast<const uint8_t*>(replacement);
  size_t written = 0;
  while (written < size) {
    const ssize_t step = pwrite(fd, bytes + written, size - written,
                                static_cast<off_t>(addr + written));
    if (step < 0 && errno == EINTR) continue;
    if (step <= 0) {
      close(fd);
      LOGE("inlineHook: patching %p stopped after %zu of %zu byte(s)",
           reinterpret_cast<const void*>(addr), written, size);
      return written == 0 ? 0 : 2;
    }
    written += static_cast<size_t>(step);
  }
  close(fd);

  return memcmp(reinterpret_cast<const void*>(addr), replacement, size) == 0
             ? 1
             : 2;
}

#endif

namespace znn::hook::backend {

#if defined(__aarch64__)

namespace {

constexpr size_t kPatchWindow = 8;

std::mutex g_mutex;
std::map<uintptr_t, yuki::ihook::Hook> g_records;

bool patchWindowFree(uintptr_t address) {
  for (const auto& entry : g_records) {
    const uintptr_t target = entry.first;
    if (!yuki::ihook::hook_ranges_overlap(address, kPatchWindow, target,
                                          kPatchWindow))
      continue;
    LOGE("inlineHook %p failed: it is inside the patch window of %p",
         reinterpret_cast<void*>(address), reinterpret_cast<void*>(target));
    return false;
  }
  return true;
}

bool singleThreaded(const char* what, void* target) {
  if (yuki::ihook::single_threaded_process()) return true;
  LOGE("inline%s %p refused: this process has more than one thread", what,
       target);
  return false;
}

}  // namespace

bool yukiihookHook(void* target, void* replacement, void** original) {
  if (!target || !replacement) return false;

  const uintptr_t address = reinterpret_cast<uintptr_t>(target);
  std::lock_guard<std::mutex> lk(g_mutex);

  if (g_records.find(address) != g_records.end()) {
    LOGE("inlineHook %p failed: already hooked", target);
    return false;
  }
  if (!patchWindowFree(address)) return false;
  if (!singleThreaded("Hook", target)) return false;

  yuki::ihook::Hook& hook = g_records[address];
  yuki::ihook::a64::Error reason = yuki::ihook::a64::Error::None;
  void* trampoline =
      yuki::ihook::install(target, replacement, &hook, false, false, &reason);
  if (!trampoline) {
    g_records.erase(address);
    LOGE("inlineHook %p failed: %s", target,
         yuki::ihook::a64::error_name(reason));
    return false;
  }

  if (original) *original = trampoline;
  return true;
}

bool yukiihookUnhook(void* target) {
  if (!target) return false;

  const uintptr_t address = reinterpret_cast<uintptr_t>(target);
  std::lock_guard<std::mutex> lk(g_mutex);

  const auto it = g_records.find(address);
  if (it == g_records.end()) {
    LOGE("inlineUnhook %p failed: not hooked", target);
    return false;
  }
  if (!singleThreaded("Unhook", target)) return false;

  if (!yuki::ihook::uninstall(&it->second)) {
    LOGE("inlineUnhook %p failed", target);
    return false;
  }
  g_records.erase(it);
  return true;
}

#else

bool yukiihookHook(void* target, void* replacement, void** original) {
  (void)target;
  (void)replacement;
  (void)original;
  return false;
}

bool yukiihookUnhook(void* target) {
  (void)target;
  return false;
}

#endif

}  // namespace znn::hook::backend
