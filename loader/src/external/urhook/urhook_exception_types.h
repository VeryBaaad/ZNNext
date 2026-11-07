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

#include <stdexcept>

namespace std {
namespace znn_urhook {

class runtime_error {
 public:
  explicit runtime_error(const char* message) : message_(message) {}

  const char* what() const noexcept { return message_; }

 private:
  const char* message_;
};

class invalid_argument : public runtime_error {
 public:
  using runtime_error::runtime_error;
};

}  // namespace znn_urhook
}  // namespace std
