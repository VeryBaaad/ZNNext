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

#include <mutex>
#include <set>

#include "hook/backend.h"
#include "log.h"

#ifndef __riscv
#include <config.h>
#include <gum/gum.h>
#include <gum/guminterceptor.h>
#endif

namespace znn::hook::backend {

#ifndef __riscv

namespace {

std::once_flag g_init_once;
GumInterceptor* g_interceptor = nullptr;
std::mutex g_mutex;
std::set<uintptr_t> g_hooked;

const char* replaceResultName(GumReplaceReturn result) {
  switch (result) {
    case GUM_REPLACE_OK:
      return "ok";
    case GUM_REPLACE_INVALID_INSTRUCTION:
      return "invalid instruction";
    case GUM_REPLACE_WRONG_SIGNATURE:
      return "wrong signature";
    case GUM_REPLACE_ALREADY_REPLACED:
      return "already replaced";
    case GUM_REPLACE_POLICY_VIOLATION:
      return "policy violation";
    case GUM_REPLACE_WRONG_TYPE:
      return "wrong type";
  }
  return "unknown error";
}

void initFridaGum() {
  gum_init_embedded();
  g_interceptor = gum_interceptor_obtain();
  if (g_interceptor) LOGI("frida-gum engine ready");
}

}  // namespace

bool fridaGumInit() {
  std::call_once(g_init_once, initFridaGum);
  if (!g_interceptor) {
    LOGE("frida-gum initialization failed");
    return false;
  }
  return true;
}

bool fridaGumHook(void* target, void* replacement, void** original) {
  if (!fridaGumInit()) return false;

  const uintptr_t key = reinterpret_cast<uintptr_t>(target);
  std::lock_guard<std::mutex> lk(g_mutex);
  if (!g_hooked.insert(key).second) {
    LOGE("inlineHook %p failed: already hooked", target);
    return false;
  }

  const GumReplaceReturn result = gum_interceptor_replace(
      g_interceptor, target, replacement, original, nullptr);
  if (result != GUM_REPLACE_OK) {
    g_hooked.erase(key);
    LOGE("inlineHook %p failed: %s (%d)", target, replaceResultName(result),
         result);
    return false;
  }
  return true;
}

bool fridaGumUnhook(void* target) {
  if (!fridaGumInit()) return false;

  const uintptr_t key = reinterpret_cast<uintptr_t>(target);
  std::lock_guard<std::mutex> lk(g_mutex);
  const auto it = g_hooked.find(key);
  if (it == g_hooked.end()) {
    LOGE("inlineUnhook %p failed: not hooked", target);
    return false;
  }

  gum_interceptor_revert(g_interceptor, target);
  g_hooked.erase(it);
  return true;
}

#else

bool fridaGumInit() { return false; }

bool fridaGumHook(void* target, void* replacement, void** original) {
  (void)target;
  (void)replacement;
  (void)original;
  return false;
}

bool fridaGumUnhook(void* target) {
  (void)target;
  return false;
}

#endif

}  // namespace znn::hook::backend
