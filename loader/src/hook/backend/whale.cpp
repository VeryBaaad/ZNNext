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

#include "hook/backend.h"
#include "log.h"

#ifndef __riscv
#include <znn_whale.h>
#endif

namespace znn::hook::backend {

#ifndef __riscv

namespace {

std::mutex g_mutex;
std::map<uintptr_t, znn_whale_hook_t*> g_hooks;

}  // namespace

bool whaleHook(void* target, void* replacement, void** original) {
  if (!target || !replacement) return false;

  const uintptr_t key = reinterpret_cast<uintptr_t>(target);
  std::lock_guard<std::mutex> lk(g_mutex);
  if (g_hooks.find(key) != g_hooks.end()) {
    LOGE("inlineHook %p failed: already hooked", target);
    return false;
  }

  void* trampoline = nullptr;
  znn_whale_hook_t* hook =
      znn_whale_hook_create(target, replacement, &trampoline);
  if (!hook) {
    LOGE("inlineHook %p failed", target);
    return false;
  }

  g_hooks.emplace(key, hook);
  if (original) *original = trampoline;
  return true;
}

bool whaleUnhook(void* target) {
  const uintptr_t key = reinterpret_cast<uintptr_t>(target);
  std::lock_guard<std::mutex> lk(g_mutex);
  const auto it = g_hooks.find(key);
  if (it == g_hooks.end()) {
    LOGE("inlineUnhook %p failed: not hooked", target);
    return false;
  }
  znn_whale_hook_destroy(it->second);
  g_hooks.erase(it);
  return true;
}

#else

bool whaleHook(void* target, void* replacement, void** original) {
  (void)target;
  (void)replacement;
  (void)original;
  return false;
}

bool whaleUnhook(void* target) {
  (void)target;
  return false;
}

#endif

}  // namespace znn::hook::backend
