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

#ifndef ZNN_FORMAT_H
#define ZNN_FORMAT_H

#include <stdint.h>

#define ZNN_LINE_CAPACITY 80

static inline int znn_append_text(char* buffer, int length, int capacity,
                                  const char* text) {
  while (*text != '\0' && length < capacity) buffer[length++] = *text++;
  return length;
}

static inline int znn_append_u64(char* buffer, int length, int capacity,
                                 uint64_t value) {
  char digits[20];
  int count = 0;
  do {
    digits[count++] = (char)('0' + (char)(value % 10));
    value /= 10;
  } while (value != 0 && count < (int)sizeof(digits));
  while (count > 0 && length < capacity) buffer[length++] = digits[--count];
  return length;
}

static inline int znn_append_i32(char* buffer, int length, int capacity,
                                 int32_t value) {
  if (value < 0) {
    if (length < capacity) buffer[length++] = '-';
    return znn_append_u64(buffer, length, capacity,
                          (uint64_t)(-(int64_t)value));
  }
  return znn_append_u64(buffer, length, capacity, (uint64_t)value);
}

static inline int znn_format_exec_record(char* buffer, int capacity,
                                         uint64_t seq, int32_t pid,
                                         int32_t tgid, int32_t ppid) {
  int length = 0;
  length = znn_append_text(buffer, length, capacity, "seq=");
  length = znn_append_u64(buffer, length, capacity, seq);
  length = znn_append_text(buffer, length, capacity, " pid=");
  length = znn_append_i32(buffer, length, capacity, pid);
  length = znn_append_text(buffer, length, capacity, " tgid=");
  length = znn_append_i32(buffer, length, capacity, tgid);
  length = znn_append_text(buffer, length, capacity, " ppid=");
  length = znn_append_i32(buffer, length, capacity, ppid);
  length = znn_append_text(buffer, length, capacity, "\n");
  return length;
}

#endif /* ZNN_FORMAT_H */
