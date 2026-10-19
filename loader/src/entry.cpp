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

#include <unistd.h>

#include "config/debugdata.h"
#include "config/hooks.h"
#include "hyos/runtime.h"
#include "log.h"
#include "module/loader.h"
#include "module/plan.h"
#include "process/self.h"

extern "C" __attribute__((visibility("default"))) void znn_loader_init(
    const void* raw_plan) {
  znn::module::Plan plan;
  if (!znn::module::parsePlan(raw_plan, plan)) {
    LOGE("loader: no valid boot plan, doing nothing in this process");
    return;
  }

  LOGI("loader initialized in pid %d (%s)", getpid(),
       znn::process::exePath().c_str());

  znn::config::resolveHookEngines(plan.inline_engine, plan.plt_engine);
  znn::config::resolveDebugDataBackend(plan.debugdata);

  if (znn::process::exeName() == "hyos_spawner") znn::hyos::setActive(true);

  znn::module::loadAll(plan);
}
