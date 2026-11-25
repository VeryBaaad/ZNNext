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

#include <link.h>
#include <sys/mman.h>

#include <cstdint>
#include <string>
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

const MapEntry* firstMappingOfFile(const std::vector<MapEntry>& maps,
                                   const MapEntry& hit) {
  if (hit.start < hit.offset) return &hit;
  const uintptr_t load_base = hit.start - hit.offset;
  const MapEntry* first_readable = nullptr;
  for (const auto& m : maps) {
    if (m.dev != hit.dev || m.inode != hit.inode) continue;
    if (m.start < m.offset || m.start - m.offset != load_base) continue;
    if (!(m.perms & PROT_READ)) continue;
    if (m.offset == 0) return &m;
    if (!first_readable) first_readable = &m;
  }
  return first_readable ? first_readable : &hit;
}

struct LinkerPathContext {
  uintptr_t base;
  const char* path;
};

int findLinkerPath(struct dl_phdr_info* info, size_t, void* data) {
  auto* context = static_cast<LinkerPathContext*>(data);
  uintptr_t image_base = UINTPTR_MAX;
  for (int i = 0; i < info->dlpi_phnum; ++i) {
    const ElfW(Phdr) & phdr = info->dlpi_phdr[i];
    if (phdr.p_type != PT_LOAD) continue;
    const uintptr_t address =
        static_cast<uintptr_t>(info->dlpi_addr) + phdr.p_vaddr;
    if (address < image_base) image_base = address;
  }
  if (image_base == UINTPTR_MAX)
    image_base = static_cast<uintptr_t>(info->dlpi_addr);
  if (image_base != context->base) return 0;
  if (info->dlpi_name && info->dlpi_name[0]) context->path = info->dlpi_name;
  return 1;
}

std::string linkerPathForBase(uintptr_t base) {
  LinkerPathContext context{base, nullptr};
  dl_iterate_phdr(findLinkerPath, &context);
  return context.path ? context.path : "";
}

void fileRangeForTarget(const std::vector<MapEntry>& maps,
                        const MapEntry& entry, uintptr_t load_base,
                        uintptr_t* file_offset, size_t* file_size) {
  *file_offset = entry.offset;
  uintptr_t file_end = 0;
  for (const auto& m : maps) {
    if (m.dev != entry.dev || m.inode != entry.inode) continue;
    if (m.start < m.offset || m.start - m.offset != load_base) continue;
    if (m.end < load_base) continue;
    const uintptr_t mapping_end = m.end - load_base;
    if (mapping_end > file_end) file_end = mapping_end;
  }
  *file_size = file_end > *file_offset
                   ? static_cast<size_t>(file_end - *file_offset)
                   : 0;
}

}  // namespace

bool pltHook(void* base, const char* symbol, void* replacement,
             void** original) {
  if (!base || !symbol || !replacement) return false;
  ensurePltEngineReady();

  const uintptr_t b = reinterpret_cast<uintptr_t>(base);
  const auto maps = parseMaps("self");
  const MapEntry* hit = findMapping(maps, b);
  if (!hit) {
    LOGE("pltHook %s: base %p not found in maps", symbol, base);
    return false;
  }
  const MapEntry* entry = firstMappingOfFile(maps, *hit);
  if (entry->path.empty()) {
    LOGE("pltHook %s: base %p has no path in maps", symbol, base);
    return false;
  }
  const uintptr_t load_base =
      hit->start >= hit->offset ? hit->start - hit->offset : entry->start;
  const std::string linker_path = linkerPathForBase(load_base);

  backend::PltTarget target;
  target.maps_path = entry->path;
  target.linker_path = linker_path.empty() ? entry->path : linker_path;
  target.dev = entry->dev;
  target.inode = entry->inode;
  target.base = load_base;
  fileRangeForTarget(maps, *entry, load_base, &target.file_offset,
                     &target.file_size);

  LOGI("pltHook base=%p symbol=%s path=%s linker=%s dev=%llu inode=%llu", base,
       symbol, entry->path.c_str(), target.linker_path.c_str(),
       static_cast<unsigned long long>(entry->dev),
       static_cast<unsigned long long>(entry->inode));

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
