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
#include <cstring>
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

struct ElfTarget {
  uintptr_t header = 0;
  uintptr_t load_bias = 0;
  const MapEntry* mapping = nullptr;
  const ElfW(Phdr) * phdr = nullptr;
  size_t phnum = 0;
};

bool readElfTarget(uintptr_t candidate, const std::vector<MapEntry>& maps,
                   uintptr_t reference, ElfTarget* target) {
  const MapEntry* mapping = nullptr;
  for (const auto& m : maps) {
    if (m.inode == 0 || !(m.perms & PROT_READ)) continue;
    if (candidate < m.start || candidate >= m.end) continue;
    if (m.end - candidate < sizeof(ElfW(Ehdr))) continue;
    mapping = &m;
    break;
  }
  if (!mapping) return false;

  const auto* ehdr = reinterpret_cast<const ElfW(Ehdr)*>(candidate);
  if (memcmp(ehdr->e_ident, ELFMAG, SELFMAG) != 0) return false;
  if (ehdr->e_ident[EI_CLASS] != (sizeof(void*) == 8 ? ELFCLASS64 : ELFCLASS32))
    return false;
  if (ehdr->e_type != ET_DYN && ehdr->e_type != ET_EXEC) return false;
  if (ehdr->e_phentsize != sizeof(ElfW(Phdr)) || ehdr->e_phnum == 0 ||
      ehdr->e_phnum > 4096)
    return false;

  uintptr_t phdr_address = 0;
  if (__builtin_add_overflow(candidate, static_cast<uintptr_t>(ehdr->e_phoff),
                             &phdr_address))
    return false;
  uintptr_t phdr_size = 0;
  if (__builtin_mul_overflow(static_cast<uintptr_t>(ehdr->e_phnum),
                             sizeof(ElfW(Phdr)), &phdr_size))
    return false;
  if (phdr_address < mapping->start || phdr_address > mapping->end)
    return false;
  if (phdr_size > mapping->end - phdr_address) return false;

  const auto* phdr = reinterpret_cast<const ElfW(Phdr)*>(phdr_address);
  uintptr_t load_bias = 0;
  bool has_base = false;
  for (size_t i = 0; i < ehdr->e_phnum; ++i) {
    if (phdr[i].p_type != PT_LOAD || phdr[i].p_offset != 0) continue;
    if (candidate < phdr[i].p_vaddr) return false;
    load_bias = candidate - phdr[i].p_vaddr;
    has_base = true;
    break;
  }
  if (!has_base) return false;

  bool contains_reference = false;
  for (size_t i = 0; i < ehdr->e_phnum; ++i) {
    if (phdr[i].p_type != PT_LOAD) continue;
    uintptr_t start = 0;
    if (__builtin_add_overflow(load_bias, phdr[i].p_vaddr, &start)) continue;
    uintptr_t end = 0;
    if (__builtin_add_overflow(start, phdr[i].p_memsz, &end)) continue;
    if (reference >= start && reference < end) {
      contains_reference = true;
      break;
    }
  }
  if (!contains_reference) return false;

  target->header = candidate;
  target->load_bias = load_bias;
  target->mapping = mapping;
  target->phdr = phdr;
  target->phnum = ehdr->e_phnum;
  return true;
}

bool resolveElfTarget(const std::vector<MapEntry>& maps, const MapEntry& hit,
                      ElfTarget* target) {
  std::vector<uintptr_t> candidates;
  candidates.push_back(hit.start);
  if (hit.start >= hit.offset) candidates.push_back(hit.start - hit.offset);
  for (const auto& m : maps) {
    if (m.dev != hit.dev || m.inode != hit.inode) continue;
    if (!(m.perms & PROT_READ)) continue;
    candidates.push_back(m.start);
  }

  for (const uintptr_t candidate : candidates) {
    if (candidate != 0 && readElfTarget(candidate, maps, hit.start, target))
      return true;
  }
  return false;
}

struct LinkerPathContext {
  uintptr_t load_bias;
  const char* path;
};

int findLinkerPath(struct dl_phdr_info* info, size_t, void* data) {
  auto* context = static_cast<LinkerPathContext*>(data);
  if (static_cast<uintptr_t>(info->dlpi_addr) != context->load_bias) return 0;
  if (info->dlpi_name && info->dlpi_name[0]) context->path = info->dlpi_name;
  return 1;
}

std::string linkerPathForLoadBias(uintptr_t load_bias) {
  LinkerPathContext context{load_bias, nullptr};
  dl_iterate_phdr(findLinkerPath, &context);
  return context.path ? context.path : "";
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
    for (size_t i = 0; i < target.phnum; ++i) {
      const ElfW(Phdr) & phdr = target.phdr[i];
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
  const auto maps = parseMaps("self");
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
  const std::string linker_path = linkerPathForLoadBias(elf.load_bias);

  backend::PltTarget target;
  target.maps_path = elf.mapping->path;
  target.linker_path = linker_path.empty() ? elf.mapping->path : linker_path;
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
