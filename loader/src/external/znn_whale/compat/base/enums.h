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

#ifndef WHALE_BASE_ENUMS_H_
#define WHALE_BASE_ENUMS_H_

#include <cstddef>

enum class PointerSize : size_t {
  k32 = 4,
  k64 = 8,
};

static constexpr PointerSize kRuntimePointerSize =
    sizeof(void*) == 8U ? PointerSize::k64 : PointerSize::k32;

#endif  // WHALE_BASE_ENUMS_H_
