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

#pragma once

#include <pthread.h>
#include <sys/types.h>

#include <atomic>
#include <cstdint>

#include "zygisk_next_api.h"

namespace znn::hyos {

constexpr int kMaxModules = 4;

constexpr int kNameEmpty = 0;
constexpr int kNameWriting = 1;
constexpr int kNameReady = 2;

using ForkFn = pid_t (*)();
using SetcontextFn = int (*)(uid_t uid, int is_system_server,
                             const char* se_info, const char* pkg_name);
using SetnameFn = int (*)(pthread_t thread, const char* name);

struct SpawnerImage {
  dev_t dev = 0;
  ino_t inode = 0;
  uintptr_t base = 0;
};

// forked children inherit
struct State {
  bool active = false;
  ZygiskNextHyosModule modules[kMaxModules] = {};
  int module_count = 0;
  bool in_child = false;
  std::atomic<bool> fired{false};  // delivered already, once
  bool warned = false;
  bool plt_fork_tried = false;
  bool plt_ctx_tried = false;
  ForkFn orig_fork = nullptr;
  SetcontextFn orig_setcontext = nullptr;
  SetnameFn orig_setname = nullptr;
  SpawnerImage spawner{};
  char captured_name[256] = {};
  std::atomic<int> name_state{kNameEmpty};
};

State& state();

}  // namespace znn::hyos
