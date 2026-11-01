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

#include <map>
#include <memory>
#include <mutex>
#include <string>

#include "hook/backend.h"
#include "log.h"

#ifndef __riscv
#include <yukiplt.h>
#endif

namespace znn::hook::backend {

#ifndef __riscv

namespace {

struct Record {
  yukiplt_c_handle handle{};
  std::unique_ptr<void*> original_storage;
  void* original = nullptr;
};

std::mutex g_mutex;
std::map<std::string, Record> g_records;

std::string recordKey(const std::string& caller_path, const char* symbol) {
  std::string key = caller_path;
  key += '\x01';
  key += symbol;
  return key;
}

bool findImage(const std::string& caller_path, uintptr_t caller_base,
               yukiplt_c_image* image) {
  const yukiplt_c_error by_address = yukiplt_c_image_containing(
      reinterpret_cast<const void*>(caller_base), image);
  if (by_address.code == YUKIPLT_C_SUCCESS) return true;
  const yukiplt_c_error by_name =
      yukiplt_c_image_from_soname(caller_path.c_str(), image);
  if (by_name.code == YUKIPLT_C_SUCCESS) return true;
  LOGE("pltHook: cannot find %s (base %p): %s / %s", caller_path.c_str(),
       reinterpret_cast<void*>(caller_base),
       yukiplt_c_error_name(by_address.code),
       yukiplt_c_error_name(by_name.code));
  return false;
}

yukiplt_c_error hookImage(const yukiplt_c_image& image,
                          const std::string& caller_path, const char* symbol,
                          void* replacement, void** original_storage,
                          yukiplt_c_handle* handle) {
  const yukiplt_c_selector selector = YUKIPLT_C_SELECTOR(symbol);
  yukiplt_c_slot_report report{};

  yukiplt_c_named_image named{};
  named.struct_size = sizeof(named);
  named.image = image;
  named.path = caller_path.c_str();

  const yukiplt_c_error named_error = yukiplt_c_hook_named(
      &named, &selector, replacement, original_storage, handle, &report);
  if (named_error.code != YUKIPLT_C_IMAGE_INVALIDATED) return named_error;
  return yukiplt_c_hook(&image, &selector, replacement, original_storage, handle,
                        &report);
}

}  // namespace

bool yukipltHook(const std::string& caller_path, uintptr_t caller_base,
                 const char* symbol, void* replacement, void** original) {
  if (caller_path.empty() || !symbol || !replacement) return false;

  const std::string key = recordKey(caller_path, symbol);
  std::lock_guard<std::mutex> lk(g_mutex);

  if (const auto it = g_records.find(key); it != g_records.end()) {
    Record& record = it->second;
    if (replacement != record.original) {
      LOGE("pltHook %s: %s is already hooked, unhook first", symbol,
           caller_path.c_str());
      return false;
    }
    const yukiplt_c_error error = yukiplt_c_unhook(&record.handle, nullptr, 0);
    if (error.code != YUKIPLT_C_SUCCESS &&
        error.code != YUKIPLT_C_IMAGE_INVALIDATED) {
      LOGE("pltHook %s: cannot restore slot in %s: %s", symbol,
           caller_path.c_str(), yukiplt_c_error_name(error.code));
      return false;
    }
    if (error.code == YUKIPLT_C_IMAGE_INVALIDATED)
      LOGW("pltHook %s: %s is no longer loaded, dropping the hook", symbol,
           caller_path.c_str());
    void* const previous = record.original;
    g_records.erase(it);
    if (original) *original = previous;
    return true;
  }

  yukiplt_c_image image{};
  if (!findImage(caller_path, caller_base, &image)) return false;

  auto storage = std::make_unique<void*>(nullptr);
  yukiplt_c_handle handle{};
  const yukiplt_c_error error = hookImage(image, caller_path, symbol,
                                          replacement, storage.get(), &handle);
  if (error.code != YUKIPLT_C_SUCCESS) {
    LOGE("pltHook %s in %s failed: %s", symbol, caller_path.c_str(),
         yukiplt_c_error_name(error.code));
    return false;
  }

  void* const resolved = yukiplt_c_load_original(storage.get());
  Record record;
  record.handle = handle;
  record.original = resolved;
  record.original_storage = std::move(storage);
  g_records.emplace(key, std::move(record));
  if (original) *original = resolved;
  return true;
}

#else

bool yukipltHook(const std::string& caller_path, uintptr_t caller_base,
                 const char* symbol, void* replacement, void** original) {
  (void)caller_path;
  (void)caller_base;
  (void)symbol;
  (void)replacement;
  (void)original;
  return false;
}

#endif

}  // namespace znn::hook::backend
