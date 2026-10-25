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

#include <sys/mman.h>
#include <unistd.h>
#include <ur/memory.h>

#include <cstdio>
#include <cstring>

namespace ur::memory {
namespace {

int permsToProt(const char* perms) {
  int prot = 0;
  if (perms[0] == 'r') prot |= PROT_READ;
  if (perms[1] == 'w') prot |= PROT_WRITE;
  if (perms[2] == 'x') prot |= PROT_EXEC;
  return prot;
}

}  // namespace

bool read(uintptr_t address, void* buffer, size_t size) {
  if (!address || !buffer) return false;
  memcpy(buffer, reinterpret_cast<const void*>(address), size);
  return true;
}

bool write(uintptr_t address, const void* buffer, size_t size) {
  if (!address || !buffer) return false;
  memcpy(reinterpret_cast<void*>(address), buffer, size);
  return true;
}

bool protect(uintptr_t address, size_t size, int prot) {
  const long page_size = sysconf(_SC_PAGESIZE);
  if (page_size <= 0) return false;

  const uintptr_t page_mask = static_cast<uintptr_t>(page_size) - 1;
  const uintptr_t page_start = address & ~page_mask;
  const size_t total_size = size + (address - page_start);
  const size_t aligned_size = (total_size + page_mask) & ~page_mask;
  return mprotect(reinterpret_cast<void*>(page_start), aligned_size, prot) == 0;
}

void flush_instruction_cache(uintptr_t address, size_t size) {
  char* start = reinterpret_cast<char*>(address);
  char* end = start + size;

  asm volatile("dsb ish" : : : "memory");

  long cache_line_size = sysconf(_SC_LEVEL1_ICACHE_LINESIZE);
  if (cache_line_size <= 0) cache_line_size = 64;
  for (char* p = start; p < end; p += cache_line_size) {
    asm volatile("ic ivau, %0" : : "r"(p) : "memory");
  }

  asm volatile("dsb ish" : : : "memory");
  asm volatile("isb" : : : "memory");
}

bool find_mapped_region(uintptr_t address, MappedRegion& region) {
  FILE* file = fopen("/proc/self/maps", "r");
  if (!file) return false;

  char line[1024];
  bool found = false;
  while (!found && fgets(line, sizeof(line), file)) {
    unsigned long start = 0;
    unsigned long end = 0;
    unsigned long offset = 0;
    unsigned long inode = 0;
    char perms[8] = {};
    char dev[16] = {};
    int path_offset = 0;
    if (sscanf(line, "%lx-%lx %7s %lx %15s %lu %n", &start, &end, perms,
               &offset, dev, &inode, &path_offset) < 6) {
      continue;
    }
    if (address < start || address >= end) continue;

    const char* path = line + path_offset;
    while (*path == ' ' || *path == '\t') ++path;
    size_t length = strlen(path);
    while (length > 0 &&
           (path[length - 1] == '\n' || path[length - 1] == '\r')) {
      --length;
    }

    region.start = start;
    region.end = end;
    region.offset = offset;
    region.perms = perms;
    region.path.assign(path, length);
    found = true;
  }

  fclose(file);
  return found;
}

bool atomic_patch(uintptr_t address, const uint8_t* patch_code,
                  size_t patch_size) {
  if (patch_size == 0) return true;

  int original_prot = PROT_READ | PROT_EXEC;
  MappedRegion region;
  if (find_mapped_region(address, region)) {
    const int prot = permsToProt(region.perms.c_str());
    if (prot != 0) original_prot = prot;
  }

  if (!protect(address, patch_size, PROT_READ | PROT_WRITE | PROT_EXEC)) {
    return false;
  }

  if (patch_size > 4 && !write(address + 4, patch_code + 4, patch_size - 4)) {
    protect(address, patch_size, original_prot);
    return false;
  }
  if (!write(address, patch_code, 4)) {
    protect(address, patch_size, original_prot);
    return false;
  }

  protect(address, patch_size, original_prot);
  flush_instruction_cache(address, patch_size);
  return true;
}

}  // namespace ur::memory
