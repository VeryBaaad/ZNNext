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

#include <iomanip>
#include <ostream>

#if !_LIBCPP_HAS_LOCALIZATION
namespace std {

static const int hex = 0;
static const int dec = 0;

static int setw(int) { return 0; }
static int setfill(int) { return 0; }
static int setprecision(int) { return 0; }

}  // namespace std
#endif

namespace vixl {

template <typename T>
inline std::ostream& operator<<(std::ostream& os, const T&) {
  return os;
}

namespace aarch32 {

template <typename T>
inline std::ostream& operator<<(std::ostream& os, const T&) {
  return os;
}

}  // namespace aarch32
}  // namespace vixl
