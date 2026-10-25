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

#include "znn_whale.h"

#include <cstdint>
#include <new>

#if defined(__aarch64__)
#include "dbi/arm64/inline_hook_arm64.h"
#elif defined(__arm__)
#include "dbi/arm/inline_hook_arm.h"
#elif defined(__i386__)
#include "dbi/x86/inline_hook_x86.h"
#elif defined(__x86_64__)
#include "dbi/x86_64/inline_hook_x86_64.h"
#endif

#if defined(__aarch64__) || defined(__arm__) || defined(__i386__) || \
    defined(__x86_64__)

namespace {

#if defined(__aarch64__)
using WhaleInlineHook = whale::arm64::Arm64InlineHook;
#elif defined(__arm__)
using WhaleInlineHook = whale::arm::ArmInlineHook;
#elif defined(__i386__)
using WhaleInlineHook = whale::x86::X86InlineHook;
#else
using WhaleInlineHook = whale::x86_64::X86_64InlineHook;
#endif

}  // namespace

struct znn_whale_hook {
  whale::InlineHook* impl;
};

#endif

extern "C" znn_whale_hook_t* znn_whale_hook_create(void* target,
                                                   void* replacement,
                                                   void** trampoline) {
#if defined(__aarch64__) || defined(__arm__) || defined(__i386__) || \
    defined(__x86_64__)
  if (!target || !replacement || !trampoline) return nullptr;
  *trampoline = nullptr;

  auto* holder = new (std::nothrow) znn_whale_hook{nullptr};
  if (!holder) return nullptr;

  auto* impl = new (std::nothrow)
      WhaleInlineHook(reinterpret_cast<intptr_t>(target),
                      reinterpret_cast<intptr_t>(replacement),
                      reinterpret_cast<intptr_t*>(trampoline));
  if (!impl) {
    delete holder;
    return nullptr;
  }

  impl->StartHook();
  if (*trampoline == nullptr) {
    impl->StopHook();
    delete impl;
    delete holder;
    return nullptr;
  }

  holder->impl = impl;
  return holder;
#else
  (void)target;
  (void)replacement;
  (void)trampoline;
  return nullptr;
#endif
}

extern "C" void znn_whale_hook_destroy(znn_whale_hook_t* hook) {
  if (!hook) return;
  if (hook->impl) {
    hook->impl->StopHook();
    delete hook->impl;
  }
  delete hook;
}
