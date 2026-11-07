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

#include "hyos/hooks.h"

#include <dlfcn.h>
#include <pthread.h>
#include <string.h>
#include <unistd.h>

#include <atomic>
#include <mutex>

#include "api/api.h"
#include "hook/inline.h"
#include "hook/plt.h"
#include "hyos/runtime.h"
#include "hyos/state.h"
#include "log.h"
#include "utils/elf_util.h"

namespace znn::hyos {
namespace {

std::atomic<bool> g_reinstall{false};
std::mutex g_install_mutex;

// spawner image, offset-0 mapping
void findSpawnerImage() {
  State& s = state();
  if (s.spawner.inode != 0) return;
  for (const auto& m : parseMaps("self")) {
    if (m.offset != 0 || m.inode == 0) continue;
    if (m.path.find("hyos_spawner") == std::string::npos) continue;
    s.spawner.dev = m.dev;
    s.spawner.inode = m.inode;
    s.spawner.base = m.start;
    LOGI("HYOS: spawner image dev=%llu inode=%llu base=%p",
         static_cast<unsigned long long>(m.dev),
         static_cast<unsigned long long>(m.inode),
         reinterpret_cast<void*>(m.start));
    break;
  }
}

int setcontextHook(uid_t uid, int is_system_server, const char* se_info,
                   const char* pkg_name) {
  State& s = state();
  SetcontextFn original = s.orig_setcontext.load(std::memory_order_acquire);
  if (!original) {
    original = s.fallback_setcontext.load(std::memory_order_acquire);
  }
  int ret = original ? original(uid, is_system_server, se_info, pkg_name) : -1;
  if (ret == 0) {
    deliverAppSpecialized(pkg_name, se_info);
  } else if (!s.fired && s.in_child) {
    LOGW("HYOS: selinux_android_setcontext failed (%d)", ret);
  }
  return ret;
}

int setnameHook(pthread_t thread, const char* name) {
  State& s = state();
  SetnameFn original = s.orig_setname.load(std::memory_order_acquire);
  if (!original) original = s.fallback_setname.load(std::memory_order_acquire);
  int ret = original ? original(thread, name) : -1;
  if (ret == 0 && s.in_child && !s.fired.load(std::memory_order_relaxed) &&
      thread == pthread_self() && name && name[0] != '\0') {
    int expected = kNameEmpty;
    if (s.name_state.compare_exchange_strong(expected, kNameWriting,
                                             std::memory_order_acq_rel,
                                             std::memory_order_relaxed)) {
      strlcpy(s.captured_name, name, sizeof(s.captured_name));
      s.name_state.store(kNameReady, std::memory_order_release);
      LOGI("HYOS: captured process name %s", s.captured_name);
    }
  }
  return ret;
}

pid_t forkHook() {
  State& s = state();
  ForkFn original = s.orig_fork.load(std::memory_order_acquire);
  if (!original) original = s.fallback_fork.load(std::memory_order_acquire);
  pid_t res = original ? original() : -1;
  if (res == 0) {
    atForkChild();
  } else if (res > 0 && g_reinstall.load(std::memory_order_acquire)) {
    g_reinstall.store(false, std::memory_order_release);
    if (!s.in_child) {
      installHooks();
    }
  }
  return res;
}

}  // namespace

// no allocation here
void atForkChild() {
  State& s = state();
  s.in_child = true;
  s.fired.store(false, std::memory_order_relaxed);
  s.captured_name[0] = '\0';
  s.name_state.store(kNameEmpty, std::memory_order_release);
  g_reinstall.store(false, std::memory_order_release);
}

void atForkPrepare() {
  State& s = state();
  if (!s.in_child && s.module_count.load(std::memory_order_acquire) > 0 &&
      (!s.orig_setcontext.load(std::memory_order_acquire) ||
       !s.orig_setname.load(std::memory_order_acquire))) {
    g_reinstall.store(true, std::memory_order_release);
  }
}

void resolveFallbacks() {
  State& s = state();
  if (!s.fallback_fork.load(std::memory_order_acquire)) {
    s.fallback_fork.store(reinterpret_cast<ForkFn>(dlsym(RTLD_DEFAULT, "fork")),
                          std::memory_order_release);
  }
  if (!s.fallback_setcontext.load(std::memory_order_acquire)) {
    s.fallback_setcontext.store(
        reinterpret_cast<SetcontextFn>(
            dlsym(RTLD_DEFAULT, "selinux_android_setcontext")),
        std::memory_order_release);
  }
  if (!s.fallback_setname.load(std::memory_order_acquire)) {
    s.fallback_setname.store(
        reinterpret_cast<SetnameFn>(dlsym(RTLD_DEFAULT, "pthread_setname_np")),
        std::memory_order_release);
  }
}

void installHooks() {
  State& s = state();
  std::lock_guard<std::mutex> lk(g_install_mutex);
  resolveFallbacks();
  findSpawnerImage();

  if (s.spawner.inode != 0 && !s.plt_fork_tried) {
    void* backup = nullptr;
    if (hook::pltHook(reinterpret_cast<void*>(s.spawner.base), "fork",
                      reinterpret_cast<void*>(forkHook), &backup)) {
      s.orig_fork.store(reinterpret_cast<ForkFn>(backup),
                        std::memory_order_release);
      s.plt_fork_tried = true;
      LOGI("HYOS: PLT hooked fork");
    } else {
      LOGW("HYOS: cannot hook fork, app specialization will not fire");
    }
  }
  if (s.spawner.inode != 0 && !s.plt_ctx_tried) {
    if (s.orig_setcontext.load(std::memory_order_acquire)) {
      s.plt_ctx_tried = true;
    } else {
      void* backup = nullptr;
      if (hook::pltHook(reinterpret_cast<void*>(s.spawner.base),
                        "selinux_android_setcontext",
                        reinterpret_cast<void*>(setcontextHook), &backup)) {
        s.orig_setcontext.store(reinterpret_cast<SetcontextFn>(backup),
                                std::memory_order_release);
        s.plt_ctx_tried = true;
        LOGI("HYOS: PLT hooked selinux_android_setcontext");
      }
    }
  }

  if (!s.orig_setcontext.load(std::memory_order_acquire)) {
    void* fn = dlsym(RTLD_DEFAULT, "selinux_android_setcontext");
    if (!fn) {
      void* h = dlopen("libselinux.so", RTLD_NOW);
      if (h) fn = dlsym(h, "selinux_android_setcontext");
    }
    SetcontextFn fallback =
        s.fallback_setcontext.load(std::memory_order_acquire);
    s.fallback_setcontext.store(nullptr, std::memory_order_release);
    void* backup = nullptr;
    if (fn && api::inlineHook(fn, reinterpret_cast<void*>(setcontextHook),
                              &backup) == ZN_SUCCESS) {
      s.orig_setcontext.store(reinterpret_cast<SetcontextFn>(backup),
                              std::memory_order_release);
      if (backup) {
        LOGI("HYOS: inline hooked selinux_android_setcontext");
      } else if (api::inlineUnhook(fn) == ZN_SUCCESS) {
        s.fallback_setcontext.store(fallback, std::memory_order_release);
        LOGW("HYOS: inline setcontext hook had no original and was removed");
      } else {
        LOGW(
            "HYOS: inline setcontext hook has no original and cannot be "
            "removed; replacements fail closed");
      }
    } else {
      s.fallback_setcontext.store(fallback, std::memory_order_release);
      if (fn) LOGW("HYOS: cannot inline hook selinux_android_setcontext");
    }
  }
  if (!s.orig_setname.load(std::memory_order_acquire)) {
    void* fn = dlsym(RTLD_DEFAULT, "pthread_setname_np");
    if (!fn) {
      void* h = dlopen("libc.so", RTLD_NOW);
      if (h) fn = dlsym(h, "pthread_setname_np");
    }
    SetnameFn fallback = s.fallback_setname.load(std::memory_order_acquire);
    s.fallback_setname.store(nullptr, std::memory_order_release);
    void* backup = nullptr;
    if (fn && api::inlineHook(fn, reinterpret_cast<void*>(setnameHook),
                              &backup) == ZN_SUCCESS) {
      s.orig_setname.store(reinterpret_cast<SetnameFn>(backup),
                           std::memory_order_release);
      if (backup) {
        LOGI("HYOS: inline hooked pthread_setname_np");
      } else if (api::inlineUnhook(fn) == ZN_SUCCESS) {
        s.fallback_setname.store(fallback, std::memory_order_release);
        LOGW("HYOS: inline setname hook had no original and was removed");
      } else {
        LOGW(
            "HYOS: inline setname hook has no original and cannot be removed; "
            "replacements fail closed");
      }
    } else {
      s.fallback_setname.store(fallback, std::memory_order_release);
      if (fn) LOGW("HYOS: cannot inline hook pthread_setname_np");
    }
  }
  if (!s.orig_setcontext.load(std::memory_order_acquire) && !s.warned) {
    s.warned = true;
    LOGW(
        "HYOS: selinux_android_setcontext not available, onAppSpecialized will "
        "not fire");
  }
}

}  // namespace znn::hyos
