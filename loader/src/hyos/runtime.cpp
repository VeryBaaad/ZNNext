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

#include "hyos/runtime.h"

#include <pthread.h>
#include <string.h>
#include <sys/prctl.h>

#include "hyos/hooks.h"
#include "hyos/state.h"
#include "log.h"

namespace znn::hyos {
namespace {

const ZygiskNextRuntime kRuntime = {
    .type = ZN_RUNTIME_HYOS,
    .api_version = ZYGISK_NEXT_HYOS_API_VERSION,
    .registerModule = registerModule,
};

}  // namespace

bool active() { return state().active; }

void setActive(bool value) { state().active = value; }

void deliverAppSpecialized(const char* pkg_name, const char* se_info) {
  State& s = state();
  if (!s.in_child || s.module_count == 0) return;
  bool expected = false;
  if (!s.fired.compare_exchange_strong(expected, true,
                                       std::memory_order_acq_rel,
                                       std::memory_order_relaxed)) {
    return;
  }

  static char process_name[256];
  static char package_name[256];
  static char se_info_buf[64];

  if (s.name_state.load(std::memory_order_acquire) == kNameReady) {
    strlcpy(process_name, s.captured_name, sizeof(process_name));
  } else {
    // fallback
    process_name[0] = '\0';
    if (prctl(PR_GET_NAME, process_name, 0, 0, 0) != 0 ||
        process_name[0] == '\0') {
      strlcpy(process_name, "hyos_app", sizeof(process_name));
    }
  }
  strlcpy(package_name, pkg_name ? pkg_name : "", sizeof(package_name));
  strlcpy(se_info_buf, se_info ? se_info : "", sizeof(se_info_buf));

  ZnHyosAppSpecializeArgs args = {process_name, package_name, se_info_buf};
  LOGI("HYOS app specialized: process=%s package=%s se_info=%s", process_name,
       package_name, se_info_buf);
  for (int i = 0; i < s.module_count; ++i) {
    if (s.modules[i].onAppSpecialized) {
      s.modules[i].onAppSpecialized(&args);
    }
  }
}

int registerModule(const void* module) {
  if (!module) return ZN_FAILED;
  const auto* m = static_cast<const ZygiskNextHyosModule*>(module);
  if (!m->onAppSpecialized) return ZN_FAILED;
  if (m->target_api_version > ZYGISK_NEXT_HYOS_API_VERSION) {
    LOGE("HYOS: module targets API version %d, only up to %d supported",
         m->target_api_version, ZYGISK_NEXT_HYOS_API_VERSION);
    return ZN_FAILED;
  }
  State& s = state();
  if (s.module_count >= kMaxModules) {
    LOGE("HYOS: too many modules registered (max %d)", kMaxModules);
    return ZN_FAILED;
  }
  s.modules[s.module_count++] = *m;

  if (!s.in_child) {
    pthread_atfork(atForkPrepare, nullptr, atForkChild);
  }
  installHooks();
  LOGI("HYOS: module registered (%d)", s.module_count);
  return ZN_SUCCESS;
}

const ZygiskNextRuntime* runtime() { return &kRuntime; }

}  // namespace znn::hyos
