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

#ifndef _WHALE_BASE_LOGGING_H_
#define _WHALE_BASE_LOGGING_H_

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <type_traits>

#include "base/macros.h"

#ifdef __ANDROID__
#include <android/log.h>
#endif

#define CHECK(x)                                        \
  if (UNLIKELY(!(x))) /* NOLINT*/                       \
  ::whale::LogMessageFatal(__FILE__, __LINE__).stream() \
      << "Check failed: " #x << " "

#define CHECK_OP(LHS, RHS, OP)                                      \
  for (auto _values = ::whale::MakeEagerEvaluator(LHS, RHS);        \
       UNLIKELY(!(_values.lhs OP _values.rhs));) /* NOLINT*/        \
  ::whale::LogMessageFatal(__FILE__, __LINE__).stream()             \
      << "Check failed: " << #LHS " " #OP " " #RHS << " (" #LHS "=" \
      << _values.lhs << ", " #RHS "=" << _values.rhs << ") "

#define CHECK_EQ(x, y) CHECK_OP(x, y, ==)
#define CHECK_NE(x, y) CHECK_OP(x, y, !=)
#define CHECK_LE(x, y) CHECK_OP(x, y, <=)
#define CHECK_LT(x, y) CHECK_OP(x, y, <)
#define CHECK_GE(x, y) CHECK_OP(x, y, >=)
#define CHECK_GT(x, y) CHECK_OP(x, y, >)

#define CHECK_STROP(s1, s2, sense)                          \
  if (UNLIKELY((strcmp(s1, s2) == 0) != sense)) /* NOLINT*/ \
  ::whale::LogMessageFatal(__FILE__, __LINE__).stream()     \
      << "Check failed: "                                   \
      << "\"" << s1 << "\"" << (sense ? " == " : " != ") << "\"" << s2 << "\""

#define CHECK_STREQ(s1, s2) CHECK_STROP(s1, s2, true)
#define CHECK_STRNE(s1, s2) CHECK_STROP(s1, s2, false)

#define LOG_INFO ::whale::LogMessage(__FILE__, __LINE__)
#define LOG_WARNING ::whale::LogMessage(__FILE__, __LINE__)
#define LOG_ERROR ::whale::LogMessage(__FILE__, __LINE__)
#define LOG_FATAL ::whale::LogMessageFatal(__FILE__, __LINE__)
#define LOG_QFATAL LOG_FATAL

#ifdef NDEBUG
#define LOG_DFATAL LOG_ERROR
#define DCHECK(condition) \
  while (false) CHECK(condition)
#define DCHECK_EQ(val1, val2) \
  while (false) CHECK_EQ(val1, val2)
#define DCHECK_NE(val1, val2) \
  while (false) CHECK_NE(val1, val2)
#define DCHECK_LE(val1, val2) \
  while (false) CHECK_LE(val1, val2)
#define DCHECK_LT(val1, val2) \
  while (false) CHECK_LT(val1, val2)
#define DCHECK_GE(val1, val2) \
  while (false) CHECK_GE(val1, val2)
#define DCHECK_GT(val1, val2) \
  while (false) CHECK_GT(val1, val2)
#else
#define LOG_DFATAL LOG_FATAL
#define DCHECK(condition) CHECK(condition)
#define DCHECK_EQ(val1, val2) CHECK_EQ(val1, val2)
#define DCHECK_NE(val1, val2) CHECK_NE(val1, val2)
#define DCHECK_LE(val1, val2) CHECK_LE(val1, val2)
#define DCHECK_LT(val1, val2) CHECK_LT(val1, val2)
#define DCHECK_GE(val1, val2) CHECK_GE(val1, val2)
#define DCHECK_GT(val1, val2) CHECK_GT(val1, val2)
#endif

#define LOG(severity) LOG_##severity.stream()
#define VLOG(x)  \
  if ((x) > 0) { \
  } else         \
    LOG_INFO.stream()  // NOLINT

namespace whale {

template <typename LHS, typename RHS>
struct EagerEvaluator {
  EagerEvaluator(LHS lhs, RHS rhs) : lhs(lhs), rhs(rhs) {}

  LHS lhs;
  RHS rhs;
};

template <typename LHS, typename RHS>
EagerEvaluator<LHS, RHS> MakeEagerEvaluator(LHS lhs, RHS rhs) {
  return EagerEvaluator<LHS, RHS>(lhs, rhs);
}

class LogMessage {
 public:
  LogMessage(const char* file, int line) {
    (void)file;
    (void)line;
  }

  LogMessage(const LogMessage&) = delete;
  LogMessage& operator=(const LogMessage&) = delete;

  virtual ~LogMessage() { Flush(); }

  LogMessage& stream() { return *this; }

  LogMessage& operator<<(const char* text) {
    if (text) data_ += text;
    return *this;
  }

  LogMessage& operator<<(char* text) {
    return *this << static_cast<const char*>(text);
  }

  LogMessage& operator<<(const std::string& text) {
    data_ += text;
    return *this;
  }

  LogMessage& operator<<(char value) {
    data_ += value;
    return *this;
  }

  template <typename T,
            typename std::enable_if<std::is_integral<T>::value, int>::type = 0>
  LogMessage& operator<<(T value) {
    data_ += std::to_string(value);
    return *this;
  }

  template <typename T,
            typename std::enable_if<std::is_enum<T>::value, int>::type = 0>
  LogMessage& operator<<(T value) {
    data_ += std::to_string(static_cast<long long>(value));
    return *this;
  }

  template <typename T, typename std::enable_if<
                            std::is_floating_point<T>::value, int>::type = 0>
  LogMessage& operator<<(T value) {
    data_ += std::to_string(value);
    return *this;
  }

  template <typename T>
  LogMessage& operator<<(T* value) {
    char buffer[2 + sizeof(uintptr_t) * 2 + 1];
    snprintf(
        buffer, sizeof(buffer), "0x%llx",
        static_cast<unsigned long long>(reinterpret_cast<uintptr_t>(value)));
    data_ += buffer;
    return *this;
  }

 protected:
  void Flush() {
#ifdef __ANDROID__
    __android_log_write(ANDROID_LOG_ERROR, "Whale", data_.c_str());
#else
    fwrite(data_.data(), 1, data_.size(), stderr);
#endif
  }

 private:
  std::string data_;
};

class LogMessageFatal : public LogMessage {
 public:
  LogMessageFatal(const char* file, int line) : LogMessage(file, line) {}

  LogMessageFatal(const LogMessageFatal&) = delete;
  LogMessageFatal& operator=(const LogMessageFatal&) = delete;

  ~LogMessageFatal() override {
    Flush();
    abort();
  }
};

}  // namespace whale

#endif  // _WHALE_BASE_LOGGING_H_
