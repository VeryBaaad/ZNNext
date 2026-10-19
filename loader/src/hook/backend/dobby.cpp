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

#include "hook/backend.h"
#include "log.h"

#ifndef __riscv
#include <dobby.h>
#endif

namespace znn::hook::backend {

bool dobbyHook(void* target, void* replacement, void** original) {
#ifndef __riscv
  if (DobbyHook(target, reinterpret_cast<dobby_dummy_func_t>(replacement),
                reinterpret_cast<dobby_dummy_func_t*>(original)) !=
      RS_SUCCESS) {
    LOGE("inlineHook %p failed", target);
    return false;
  }
  return true;
#else
  (void)target;
  (void)replacement;
  (void)original;
  return false;
#endif
}

bool dobbyUnhook(void* target) {
#ifndef __riscv
  return DobbyDestroy(target) == RS_SUCCESS;
#else
  (void)target;
  return false;
#endif
}

}  // namespace znn::hook::backend
