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

#include <atomic>
#include <map>
#include <memory>
#include <mutex>
#include <set>
#include <string>
#include <vector>

#include "hook/backend.h"
#include "log.h"

#ifndef __riscv
#include <bytehook.h>
#endif

namespace znn::hook::backend {

#ifndef __riscv

namespace {

struct HookContext {
  std::atomic<void*> first_original{nullptr};
  std::atomic<void*> latest_original{nullptr};
  std::atomic<int> status{BYTEHOOK_STATUS_CODE_UNINIT};
  std::atomic<void**> caller_output{nullptr};
};

std::once_flag g_init_once;
int g_init_result = -1;
std::mutex g_mutex;
std::map<std::string, void*> g_stubs;
std::map<std::string, void*> g_originals;
std::map<std::string, std::unique_ptr<HookContext>> g_contexts;
std::vector<std::unique_ptr<HookContext>> g_retired_contexts;
std::set<std::string> g_pending;

std::string hookKey(const PltTarget& target, const char* symbol) {
  std::string key = target.linker_path;
  key += '\x01';
  key += symbol;
  return key;
}

}  // namespace

extern "C" {
static void bytehookHooked(bytehook_stub_t stub, int status_code,
                           const char* caller_path_name, const char* sym_name,
                           void* new_func, void* prev_func, void* arg) {
  (void)stub;
  (void)caller_path_name;
  (void)sym_name;
  (void)new_func;
  if (!arg) return;
  auto* context = static_cast<HookContext*>(arg);
  if (status_code == BYTEHOOK_STATUS_CODE_ORIG_ADDR) {
    if (prev_func) {
      void* expected = nullptr;
      context->first_original.compare_exchange_strong(expected, prev_func);
      context->latest_original.store(prev_func, std::memory_order_release);
      if (void** output = context->caller_output.load()) *output = prev_func;
    }
    return;
  }
  context->status.store(status_code, std::memory_order_release);
  if (status_code == BYTEHOOK_STATUS_CODE_OK && prev_func)
    context->latest_original.store(prev_func, std::memory_order_release);
}
}

bool bytehookInit() {
  std::call_once(g_init_once, [] {
#if defined(__aarch64__) || defined(__arm__)
    shadowhookInit();
#endif
    g_init_result = bytehook_init(BYTEHOOK_MODE_MANUAL, false);
  });
  if (g_init_result != 0) {
    LOGE("bytehook_init failed: %d", g_init_result);
    return false;
  }
  LOGI("bytehook engine ready (mode: manual)");
  return true;
}

bool bytehookHook(const PltTarget& target, const char* symbol,
                  void* replacement, void** original) {
  if (!bytehookInit()) return false;

  const std::string& caller_path = target.linker_path;
  const std::string key = hookKey(target, symbol);
  {
    std::lock_guard<std::mutex> lk(g_mutex);
    auto stub_it = g_stubs.find(key);
    if (stub_it != g_stubs.end()) {
      void* first = g_originals[key];
      void* latest = g_originals[key];
      auto context_it = g_contexts.find(key);
      if (context_it != g_contexts.end()) {
        first =
            context_it->second->first_original.load(std::memory_order_acquire);
        latest =
            context_it->second->latest_original.load(std::memory_order_acquire);
      }
      // unhook by passing original
      if (replacement == first || replacement == latest) {
        bytehook_unhook(stub_it->second);
        if (context_it != g_contexts.end()) {
          g_retired_contexts.push_back(std::move(context_it->second));
          g_contexts.erase(context_it);
        }
        g_originals.erase(key);
        g_stubs.erase(stub_it);
        return true;
      }
      LOGE("pltHook %s: %s is already hooked, unhook first", symbol,
           caller_path.c_str());
      return false;
    }
    if (g_pending.find(key) != g_pending.end()) {
      LOGE("pltHook %s: %s is being installed", symbol, caller_path.c_str());
      return false;
    }
    g_pending.insert(key);
  }

  auto context = std::make_unique<HookContext>();
  context->caller_output.store(original);
  const bytehook_stub_t stub =
      bytehook_hook_single(caller_path.c_str(), nullptr, symbol, replacement,
                           bytehookHooked, context.get());
  if (!stub) {
    context->caller_output.store(nullptr);
    {
      std::lock_guard<std::mutex> lk(g_mutex);
      g_pending.erase(key);
      g_retired_contexts.push_back(std::move(context));
    }
    LOGE("pltHook %s: bytehook_hook_single failed for %s", symbol,
         caller_path.c_str());
    return false;
  }

  context->caller_output.store(nullptr);
  void* resolved = context->latest_original.load(std::memory_order_acquire);
  const int status = context->status.load(std::memory_order_acquire);
  if (status != BYTEHOOK_STATUS_CODE_OK || !resolved) {
    LOGE("pltHook %s: no matching relocation in %s", symbol,
         caller_path.c_str());
    bytehook_unhook(stub);
    {
      std::lock_guard<std::mutex> lk(g_mutex);
      g_pending.erase(key);
      g_retired_contexts.push_back(std::move(context));
    }
    return false;
  }

  {
    std::lock_guard<std::mutex> lk(g_mutex);
    g_pending.erase(key);
    g_stubs[key] = stub;
    g_originals[key] = resolved;
    g_contexts[key] = std::move(context);
  }
  return true;
}

#else

bool bytehookInit() { return false; }

bool bytehookHook(const PltTarget& target, const char* symbol,
                  void* replacement, void** original) {
  (void)target;
  (void)symbol;
  (void)replacement;
  (void)original;
  return false;
}

#endif

}  // namespace znn::hook::backend
