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

#include <ur/capi.h>
#include <ur/inline_hook.h>

#include <memory>
#include <new>

struct ur_inline_hook {
  std::unique_ptr<ur::inline_hook::Hook> hook;
};

extern "C" {

ur_status_t ur_inline_hook_create(uintptr_t target, void* callback,
                                  int enable_now, ur_inline_hook_t** out) {
  if (!out) return UR_STATUS_INVALID_ARG;
  *out = nullptr;
  if (!target || !callback) return UR_STATUS_INVALID_ARG;

  auto* holder = new (std::nothrow) ur_inline_hook;
  if (!holder) return UR_STATUS_ERROR;

  try {
    holder->hook = std::make_unique<ur::inline_hook::Hook>(
        target, reinterpret_cast<ur::inline_hook::Hook::Callback>(callback),
        enable_now != 0);
  } catch (...) {
    delete holder;
    return UR_STATUS_ERROR;
  }

  if (!holder->hook->is_valid() || holder->hook->get_trampoline() == 0) {
    delete holder;
    return UR_STATUS_ERROR;
  }

  *out = holder;
  return UR_STATUS_OK;
}

void ur_inline_hook_destroy(ur_inline_hook_t* hook) { delete hook; }

int ur_inline_hook_is_valid(const ur_inline_hook_t* hook) {
  return hook && hook->hook && hook->hook->is_valid() ? 1 : 0;
}

ur_status_t ur_inline_hook_enable(ur_inline_hook_t* hook) {
  if (!hook || !hook->hook) return UR_STATUS_INVALID_ARG;
  return hook->hook->enable() ? UR_STATUS_OK : UR_STATUS_ERROR;
}

ur_status_t ur_inline_hook_disable(ur_inline_hook_t* hook) {
  if (!hook || !hook->hook) return UR_STATUS_INVALID_ARG;
  return hook->hook->disable() ? UR_STATUS_OK : UR_STATUS_ERROR;
}

ur_status_t ur_inline_hook_unhook(ur_inline_hook_t* hook) {
  if (!hook || !hook->hook) return UR_STATUS_INVALID_ARG;
  hook->hook->unhook();
  return UR_STATUS_OK;
}

void* ur_inline_hook_get_trampoline(const ur_inline_hook_t* hook) {
  if (!hook || !hook->hook) return nullptr;
  return reinterpret_cast<void*>(hook->hook->get_trampoline());
}

ur_status_t ur_inline_hook_set_detour(ur_inline_hook_t* hook, void* callback) {
  if (!hook || !hook->hook || !callback) return UR_STATUS_INVALID_ARG;
  hook->hook->set_detour(
      reinterpret_cast<ur::inline_hook::Hook::Callback>(callback));
  return UR_STATUS_OK;
}

}  // extern "C"
