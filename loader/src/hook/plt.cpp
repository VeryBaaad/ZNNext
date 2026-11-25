/*
 * SPDX-License-Identifier: GPL-3.0-or-later AND LicenseRef-DCLPL-2.0
 *
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

#include "hook/plt.h"

#include <dlfcn.h>
#include <link.h>
#include <sys/mman.h>

#include <cstdint>
#include <cstring>
#include <string>
#include <utility>
#include <vector>

#include "hook/backend.h"
#include "hook/engine.h"
#include "log.h"
#include "utils/elf_util.h"

namespace znn::hook {
namespace {

const MapEntry* findMapping(const std::vector<MapEntry>& maps, uintptr_t base) {
  for (const auto& m : maps) {
    if (m.inode == 0) continue;
    if (base >= m.start && base < m.end) return &m;
    if (m.start >= m.offset && base == m.start - m.offset) return &m;
  }
  return nullptr;
}

class DlopenPin {
 public:
  explicit DlopenPin(const std::string& path) {
    if (!path.empty()) handle_ = dlopen(path.c_str(), RTLD_NOW | RTLD_NOLOAD);
  }
  ~DlopenPin() {
    if (handle_) dlclose(handle_);
  }
  bool valid() const { return handle_ != nullptr; }

 private:
  void* handle_ = nullptr;
};

struct ElfTarget {
  uintptr_t header = 0;
  uintptr_t load_bias = 0;
  std::string name;
  std::vector<ElfW(Phdr)> phdrs;
  const MapEntry* mapping = nullptr;
};

struct ElfTargetContext {
  uintptr_t reference;
  ElfTarget* target;
  bool found;
};

int findElfTarget(struct dl_phdr_info* info, size_t, void* data) {
  auto* context = static_cast<ElfTargetContext*>(data);
  if (!info->dlpi_phdr) return 0;

  for (int i = 0; i < info->dlpi_phnum; ++i) {
    const ElfW(Phdr) & phdr = info->dlpi_phdr[i];
    if (phdr.p_type != PT_LOAD) continue;
    uintptr_t start = 0;
    if (__builtin_add_overflow(static_cast<uintptr_t>(info->dlpi_addr),
                               phdr.p_vaddr, &start))
      continue;
    uintptr_t end = 0;
    if (__builtin_add_overflow(start, phdr.p_memsz, &end)) continue;
    if (context->reference < start || context->reference >= end) continue;

    context->target->load_bias = static_cast<uintptr_t>(info->dlpi_addr);
    context->target->phdrs.assign(info->dlpi_phdr,
                                  info->dlpi_phdr + info->dlpi_phnum);
    if (info->dlpi_name) context->target->name = info->dlpi_name;

    uintptr_t first_vaddr = 0;
    bool have_first = false;
    for (int j = 0; j < info->dlpi_phnum; ++j) {
      const ElfW(Phdr) & load = info->dlpi_phdr[j];
      if (load.p_type != PT_LOAD) continue;
      if (!have_first) {
        first_vaddr = load.p_vaddr;
        have_first = true;
      }
      if (load.p_offset == 0) {
        context->target->header =
            static_cast<uintptr_t>(info->dlpi_addr) + load.p_vaddr;
        break;
      }
    }
    if (context->target->header == 0 && have_first)
      context->target->header =
          static_cast<uintptr_t>(info->dlpi_addr) + first_vaddr;
    context->found = context->target->header != 0;
    return 1;
  }
  return 0;
}

bool resolveElfTarget(const std::vector<MapEntry>& maps, const MapEntry& hit,
                      ElfTarget* target) {
  ElfTargetContext context{hit.start, target, false};
  dl_iterate_phdr(findElfTarget, &context);
  if (!context.found || !target->header) return false;

  for (const auto& m : maps) {
    if (m.inode == 0) continue;
    if (target->header < m.start || target->header >= m.end) continue;
    target->mapping = &m;
    break;
  }
  return target->mapping != nullptr;
}

void fileRangeForTarget(const std::vector<MapEntry>& maps,
                        const ElfTarget& target, uintptr_t* file_offset,
                        size_t* file_size) {
  *file_offset = target.mapping->offset;
  uintptr_t file_end = 0;
  for (const auto& m : maps) {
    if (m.dev != target.mapping->dev || m.inode != target.mapping->inode)
      continue;
    if (m.offset < *file_offset) continue;

    bool overlaps_load = false;
    for (const ElfW(Phdr) & phdr : target.phdrs) {
      if (phdr.p_type != PT_LOAD) continue;
      uintptr_t start = 0;
      if (__builtin_add_overflow(target.load_bias, phdr.p_vaddr, &start))
        continue;
      uintptr_t end = 0;
      if (__builtin_add_overflow(start, phdr.p_memsz, &end)) continue;
      if (m.start < end && m.end > start) {
        overlaps_load = true;
        break;
      }
    }
    if (!overlaps_load) continue;

    const uintptr_t mapping_end = (m.offset - *file_offset) + (m.end - m.start);
    if (mapping_end > file_end) file_end = mapping_end;
  }
  *file_size = static_cast<size_t>(file_end);
}

}  // namespace

bool pltHook(void* base, const char* symbol, void* replacement,
             void** original) {
  if (!base || !symbol || !replacement) return false;
  ensurePltEngineReady();

  const uintptr_t b = reinterpret_cast<uintptr_t>(base);
  auto maps = parseMaps("self");
  const MapEntry* hit = findMapping(maps, b);
  if (!hit) {
    LOGE("pltHook %s: base %p not found in maps", symbol, base);
    return false;
  }
  ElfTarget elf;
  if (!resolveElfTarget(maps, *hit, &elf)) {
    LOGE("pltHook %s: cannot resolve the ELF header for base %p", symbol, base);
    return false;
  }
  if (elf.mapping->path.empty()) {
    LOGE("pltHook %s: base %p has no path in maps", symbol, base);
    return false;
  }

  DlopenPin pin(elf.name.empty() ? elf.mapping->path : elf.name);
  if (pin.valid()) {
    maps = parseMaps("self");
    hit = findMapping(maps, b);
    if (!hit) {
      LOGE("pltHook %s: base %p disappeared while pinning", symbol, base);
      return false;
    }
    ElfTarget refreshed;
    if (!resolveElfTarget(maps, *hit, &refreshed)) {
      LOGE("pltHook %s: target %p became invalid while pinning", symbol, base);
      return false;
    }
    elf = std::move(refreshed);
  }

  backend::PltTarget target;
  target.maps_path = elf.mapping->path;
  target.linker_path = elf.name.empty() ? elf.mapping->path : elf.name;
  target.dev = elf.mapping->dev;
  target.inode = elf.mapping->inode;
  target.base = elf.header;
  target.load_bias = elf.load_bias;
  fileRangeForTarget(maps, elf, &target.file_offset, &target.file_size);

  LOGI("pltHook base=%p symbol=%s path=%s linker=%s dev=%llu inode=%llu", base,
       symbol, elf.mapping->path.c_str(), target.linker_path.c_str(),
       static_cast<unsigned long long>(elf.mapping->dev),
       static_cast<unsigned long long>(elf.mapping->inode));

  switch (pltEngine()) {
    case PltEngine::kLsplt:
      return backend::lspltHook(target, symbol, replacement, original);
    case PltEngine::kByteHook:
      return backend::bytehookHook(target, symbol, replacement, original);
    case PltEngine::kXHook:
      return backend::xhookHook(target, symbol, replacement, original);
    case PltEngine::kPlti:
      return backend::pltiHook(target, symbol, replacement, original);
    case PltEngine::kPltHook:
      return backend::plthookHook(target, symbol, replacement, original);
    case PltEngine::kUrHook:
      return backend::urhookPltHook(target, symbol, replacement, original);
    case PltEngine::kBaaadHook:
      return backend::baaadhookPltHook(target, symbol, replacement, original);
    case PltEngine::kYukiPlt:
      return backend::yukipltHook(target, symbol, replacement, original);
    case PltEngine::kProfilo:
      return backend::profiloHook(target, symbol, replacement, original);
    case PltEngine::kSrxHook:
      return backend::srxhookPltHook(target, symbol, replacement, original);
    case PltEngine::kDisable:
      return false;
  }
  return false;
}

}  // namespace znn::hook
