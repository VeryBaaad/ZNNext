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

#include <link.h>
#include <sys/mman.h>
#include <unistd.h>
#include <ur/capi.h>
#include <ur/elf_parser.h>
#include <ur/memory.h>

#include <cstring>
#include <map>
#include <memory>
#include <mutex>
#include <new>
#include <string>

namespace {

constexpr uint32_t kAArch64JumpSlot = 1026;

int permsToProt(const std::string& perms) {
  int prot = 0;
  if (perms.size() > 0 && perms[0] == 'r') prot |= PROT_READ;
  if (perms.size() > 1 && perms[1] == 'w') prot |= PROT_WRITE;
  if (perms.size() > 2 && perms[2] == 'x') prot |= PROT_EXEC;
  return prot;
}

struct Entry {
  uintptr_t got_address = 0;
  void* original = nullptr;
  void* replacement = nullptr;
};

bool writeGot(uintptr_t address, void* value, void** previous) {
  void* old_value = nullptr;
  if (!ur::memory::read(address, &old_value, sizeof(old_value))) return false;
  if (previous) *previous = old_value;

  int old_prot = PROT_READ;
  ur::memory::MappedRegion region;
  if (ur::memory::find_mapped_region(address, region)) {
    old_prot = permsToProt(region.perms);
    if (old_prot == 0) old_prot = PROT_READ;
  }

  if (!ur::memory::protect(address, sizeof(value), PROT_READ | PROT_WRITE)) {
    return false;
  }
  const bool written = ur::memory::write(address, &value, sizeof(value));
  ur::memory::protect(address, sizeof(value), old_prot);
  return written;
}

}  // namespace

struct ur_plthook {
  uintptr_t base = 0;
  std::unique_ptr<ur::elf_parser::ElfParser> elf;
  bool parsed = false;
  std::mutex mutex;
  std::map<std::string, Entry> entries;
};

namespace {

bool parseElf(ur_plthook_t* hook) {
  if (hook->parsed && hook->elf) return true;
  if (hook->base == 0) return false;

  auto* parser = new (std::nothrow) ur::elf_parser::ElfParser(hook->base);
  if (!parser) return false;
  hook->elf.reset(parser);
  hook->parsed = parser->parse();
  return hook->parsed;
}

bool findGotAddress(ur_plthook_t* hook, const char* symbol,
                    uintptr_t* got_address) {
  if (!parseElf(hook)) return false;

  const Elf64_Sym* dynsym = hook->elf->get_dynamic_symbol_table();
  const char* dynstr = hook->elf->get_dynamic_string_table();
  const uintptr_t rel_location = hook->elf->get_plt_rel_location();
  const size_t rel_size = hook->elf->get_plt_rel_size();
  const int rel_type = hook->elf->get_plt_rel_entry_type();
  if (!dynsym || !dynstr || rel_location == 0 || rel_size == 0) return false;

  const uintptr_t load_bias = hook->elf->get_load_bias();
  const auto symbolMatches = [&](uint32_t index) {
    const char* name = dynstr + dynsym[index].st_name;
    return name && strcmp(name, symbol) == 0;
  };

  if (rel_type == DT_RELA) {
    const auto* rela = reinterpret_cast<const Elf64_Rela*>(rel_location);
    const size_t count = rel_size / sizeof(Elf64_Rela);
    for (size_t i = 0; i < count; ++i) {
      const Elf64_Rela& entry = rela[i];
      if (ELF64_R_TYPE(entry.r_info) != kAArch64JumpSlot) continue;
      if (!symbolMatches(ELF64_R_SYM(entry.r_info))) continue;
      *got_address = load_bias + entry.r_offset;
      return true;
    }
  } else if (rel_type == DT_REL) {
    const auto* rel = reinterpret_cast<const Elf64_Rel*>(rel_location);
    const size_t count = rel_size / sizeof(Elf64_Rel);
    for (size_t i = 0; i < count; ++i) {
      const Elf64_Rel& entry = rel[i];
      if (ELF64_R_TYPE(entry.r_info) != kAArch64JumpSlot) continue;
      if (!symbolMatches(ELF64_R_SYM(entry.r_info))) continue;
      *got_address = load_bias + entry.r_offset;
      return true;
    }
  }

  return false;
}

ur_status_t createHook(uintptr_t base, ur_plthook_t** out) {
  auto* hook = new (std::nothrow) ur_plthook_t;
  if (!hook) return UR_STATUS_ERROR;
  hook->base = base;
  if (base != 0) parseElf(hook);
  *out = hook;
  return UR_STATUS_OK;
}

struct ModuleSearch {
  const char* fragment = nullptr;
  uintptr_t base = 0;
};

int findModule(dl_phdr_info* info, size_t, void* data) {
  auto* search = static_cast<ModuleSearch*>(data);
  if (!info->dlpi_name || info->dlpi_name[0] == '\0') return 0;
  if (!strstr(info->dlpi_name, search->fragment)) return 0;

  uintptr_t min_vaddr = UINTPTR_MAX;
  for (ElfW(Half) i = 0; i < info->dlpi_phnum; ++i) {
    const ElfW(Phdr) & phdr = info->dlpi_phdr[i];
    if (phdr.p_type == PT_LOAD && phdr.p_vaddr < min_vaddr) {
      min_vaddr = phdr.p_vaddr;
    }
  }
  if (min_vaddr == UINTPTR_MAX) min_vaddr = 0;

  const long page_size = sysconf(_SC_PAGESIZE);
  const uintptr_t page_mask =
      page_size > 0 ? static_cast<uintptr_t>(page_size) - 1 : 0xFFF;
  search->base = info->dlpi_addr + (min_vaddr & ~page_mask);
  return 1;
}

}  // namespace

extern "C" {

ur_status_t ur_plthook_create_from_base(uintptr_t base, ur_plthook_t** out) {
  if (!out) return UR_STATUS_INVALID_ARG;
  *out = nullptr;
  return createHook(base, out);
}

ur_status_t ur_plthook_create_from_path(const char* so_path,
                                        ur_plthook_t** out) {
  if (!so_path || !out) return UR_STATUS_INVALID_ARG;
  *out = nullptr;

  ModuleSearch search;
  search.fragment = so_path;
  dl_iterate_phdr(findModule, &search);
  return createHook(search.base, out);
}

void ur_plthook_destroy(ur_plthook_t* hook) {
  if (!hook) return;
  {
    std::lock_guard<std::mutex> lock(hook->mutex);
    for (const auto& item : hook->entries) {
      writeGot(item.second.got_address, item.second.original, nullptr);
    }
    hook->entries.clear();
  }
  delete hook;
}

int ur_plthook_is_valid(const ur_plthook_t* hook) {
  return hook && hook->parsed ? 1 : 0;
}

ur_status_t ur_plthook_hook_symbol(ur_plthook_t* hook, const char* symbol,
                                   void* replacement, void** original_out) {
  if (!hook || !symbol || !replacement) return UR_STATUS_INVALID_ARG;
  std::lock_guard<std::mutex> lock(hook->mutex);

  const auto existing = hook->entries.find(symbol);
  if (existing != hook->entries.end()) {
    if (!writeGot(existing->second.got_address, replacement, nullptr)) {
      return UR_STATUS_ERROR;
    }
    existing->second.replacement = replacement;
    if (original_out) *original_out = existing->second.original;
    return UR_STATUS_OK;
  }

  uintptr_t got_address = 0;
  if (!findGotAddress(hook, symbol, &got_address)) return UR_STATUS_ERROR;

  void* original = nullptr;
  if (!writeGot(got_address, replacement, &original)) return UR_STATUS_ERROR;

  hook->entries.emplace(symbol, Entry{got_address, original, replacement});
  if (original_out) *original_out = original;
  return UR_STATUS_OK;
}

ur_status_t ur_plthook_unhook_symbol(ur_plthook_t* hook, const char* symbol) {
  if (!hook || !symbol) return UR_STATUS_INVALID_ARG;
  std::lock_guard<std::mutex> lock(hook->mutex);

  const auto it = hook->entries.find(symbol);
  if (it == hook->entries.end()) return UR_STATUS_ERROR;
  if (!writeGot(it->second.got_address, it->second.original, nullptr)) {
    return UR_STATUS_ERROR;
  }
  hook->entries.erase(it);
  return UR_STATUS_OK;
}

}  // extern "C"
