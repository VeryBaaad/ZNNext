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

#include <map>
#include <memory>
#include <mutex>
#include <string>
#include <vector>

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

struct Recovery {
  yukiplt_c_handle handle{};
  std::unique_ptr<void*> original_storage;
};

std::mutex g_mutex;
std::map<std::string, Record> g_records;
std::vector<Recovery> g_recovery;
std::vector<std::unique_ptr<void*>> g_retired_storage;

std::string recordKey(const PltTarget& target, const char* symbol) {
  std::string key = std::to_string(target.base);
  key += '\x01';
  key += target.linker_path;
  key += '\x01';
  key += symbol;
  return key;
}

void recoverLocked() {
  for (auto it = g_recovery.begin(); it != g_recovery.end();) {
    const yukiplt_c_error error = yukiplt_c_unhook(&it->handle, nullptr, 0);
    if (error.code == YUKIPLT_C_SUCCESS ||
        error.code == YUKIPLT_C_IMAGE_INVALIDATED) {
      LOGW("pltHook: recovered a retained YukiPLT handle");
      if (it->original_storage)
        g_retired_storage.push_back(std::move(it->original_storage));
      it = g_recovery.erase(it);
      continue;
    }
    LOGE("pltHook: retained YukiPLT handle is still unrecoverable: %s",
         yukiplt_c_error_name(error.code));
    ++it;
  }
}

bool copyFirstImage(const yukiplt_c_image_list& list, yukiplt_c_image* image,
                    std::string* path) {
  size_t count = 0;
  size_t text_size = 0;
  yukiplt_c_error error = yukiplt_c_image_list_copy(&list, nullptr, 0, nullptr,
                                                    0, &count, &text_size);
  if (error.code != YUKIPLT_C_SUCCESS || count == 0 || text_size == 0)
    return false;

  std::vector<yukiplt_c_image_record> records(count);
  std::vector<char> text(text_size);
  error =
      yukiplt_c_image_list_copy(&list, records.data(), records.size(),
                                text.data(), text.size(), &count, &text_size);
  if (error.code != YUKIPLT_C_SUCCESS || count == 0) return false;

  *image = records[0].image;
  path->assign(text.data() + records[0].path.offset, records[0].path.length);
  return true;
}

bool findImage(const PltTarget& target, yukiplt_c_image* image,
               std::string* path) {
  yukiplt_c_image_list list{};
  const yukiplt_c_error by_address = yukiplt_c_find_image_containing(
      reinterpret_cast<const void*>(target.base), &list);
  if (by_address.code == YUKIPLT_C_SUCCESS) {
    const bool copied = copyFirstImage(list, image, path);
    yukiplt_c_image_list_destroy(&list);
    if (copied) return true;
    LOGE("pltHook: cannot copy the image path for %s (base %p)",
         target.linker_path.c_str(), reinterpret_cast<void*>(target.base));
  }

  if (!target.linker_path.empty()) {
    const yukiplt_c_error by_name =
        yukiplt_c_image_from_soname(target.linker_path.c_str(), image);
    if (by_name.code == YUKIPLT_C_SUCCESS) {
      *path = target.linker_path;
      return true;
    }
  }
  if (!target.maps_path.empty() && target.maps_path != target.linker_path) {
    const yukiplt_c_error by_name =
        yukiplt_c_image_from_soname(target.maps_path.c_str(), image);
    if (by_name.code == YUKIPLT_C_SUCCESS) {
      *path = target.maps_path;
      return true;
    }
  }
  LOGE("pltHook: cannot find %s (base %p): %s", target.linker_path.c_str(),
       reinterpret_cast<void*>(target.base),
       yukiplt_c_error_name(by_address.code));
  return false;
}

yukiplt_c_error hookImage(const yukiplt_c_image& image,
                          const std::string& image_path, const char* symbol,
                          void* replacement, void** original_storage,
                          yukiplt_c_handle* handle) {
  const yukiplt_c_selector selector = YUKIPLT_C_SELECTOR(symbol);
  yukiplt_c_slot_report report{};

  yukiplt_c_named_image named{};
  named.struct_size = sizeof(named);
  named.image = image;
  named.path = image_path.c_str();

  return yukiplt_c_hook_named(&named, &selector, replacement, original_storage,
                              handle, &report);
}

}  // namespace

bool yukipltHook(const PltTarget& target, const char* symbol, void* replacement,
                 void** original) {
  const std::string& caller_path = target.linker_path;
  if (caller_path.empty() || !symbol || !replacement) return false;

  const std::string key = recordKey(target, symbol);
  std::lock_guard<std::mutex> lk(g_mutex);
  recoverLocked();

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
    if (record.original_storage)
      g_retired_storage.push_back(std::move(record.original_storage));
    g_records.erase(it);
    if (original) *original = previous;
    return true;
  }

  yukiplt_c_image image{};
  std::string image_path;
  if (!findImage(target, &image, &image_path)) return false;

  std::unique_ptr<void*> private_storage;
  void** storage = original;
  if (!storage) {
    private_storage = std::make_unique<void*>(nullptr);
    storage = private_storage.get();
  }

  yukiplt_c_handle handle{};
  const yukiplt_c_error error =
      hookImage(image, image_path, symbol, replacement, storage, &handle);
  if (error.code != YUKIPLT_C_SUCCESS) {
    if (handle.token != 0) {
      if (error.code == YUKIPLT_C_INDETERMINATE) {
        g_recovery.push_back(Recovery{handle, std::move(private_storage)});
        LOGE("pltHook %s in %s: %s, retained the handle for recovery", symbol,
             caller_path.c_str(), yukiplt_c_error_name(error.code));
      } else {
        const yukiplt_c_error cleanup = yukiplt_c_unhook(&handle, nullptr, 0);
        if (cleanup.code != YUKIPLT_C_SUCCESS &&
            cleanup.code != YUKIPLT_C_IMAGE_INVALIDATED) {
          g_recovery.push_back(Recovery{handle, std::move(private_storage)});
          LOGE("pltHook %s in %s: cleanup failed, retained the handle", symbol,
               caller_path.c_str());
        }
      }
    } else {
      LOGE("pltHook %s in %s failed: %s", symbol, caller_path.c_str(),
           yukiplt_c_error_name(error.code));
    }
    return false;
  }

  void* const resolved = yukiplt_c_load_original(storage);
  Record record;
  record.handle = handle;
  record.original = resolved;
  record.original_storage = std::move(private_storage);
  g_records.emplace(key, std::move(record));
  if (original) *original = resolved;
  return true;
}

#else

bool yukipltHook(const PltTarget& target, const char* symbol, void* replacement,
                 void** original) {
  (void)target;
  (void)symbol;
  (void)replacement;
  (void)original;
  return false;
}

#endif

}  // namespace znn::hook::backend
