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

#include <__fwd/sstream.h>

#include <cstdint>
#include <memory>
#include <string>
#include <type_traits>

_LIBCPP_BEGIN_NAMESPACE_STD

struct ur_hex_manip {};

inline constexpr ur_hex_manip hex{};

template <class _CharT, class _Traits, class _Allocator>
class basic_stringstream {
  static_assert(is_same_v<_CharT, char>, "only char streams are implemented");

 public:
  using string_type = basic_string<_CharT, _Traits, _Allocator>;

  basic_stringstream() = default;

  basic_stringstream& operator<<(const char* text) {
    data_ += text;
    return *this;
  }

  basic_stringstream& operator<<(const string_type& text) {
    data_ += text;
    return *this;
  }

  basic_stringstream& operator<<(char value) {
    data_ += value;
    return *this;
  }

  template <typename T, typename = enable_if_t<is_integral_v<T>>>
  basic_stringstream& operator<<(T value) {
    if (hex_) {
      appendHex(value);
    } else {
      data_ += to_string(value);
    }
    return *this;
  }

  basic_stringstream& operator<<(const ur_hex_manip&) {
    hex_ = true;
    return *this;
  }

  string_type str() const { return data_; }

 private:
  template <typename T>
  void appendHex(T value) {
    using Unsigned = make_unsigned_t<T>;
    Unsigned remaining = static_cast<Unsigned>(value);
    char buffer[sizeof(Unsigned) * 2 + 1];
    int index = sizeof(Unsigned) * 2;
    buffer[index] = '\0';
    do {
      const int digit = static_cast<int>(remaining & 0xF);
      buffer[--index] =
          static_cast<char>(digit < 10 ? '0' + digit : 'a' + digit - 10);
      remaining >>= 4;
    } while (remaining != 0);
    data_ += &buffer[index];
  }

  string_type data_;
  bool hex_ = false;
};

_LIBCPP_END_NAMESPACE_STD
