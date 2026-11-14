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

#include "api/api.h"
#include "log.h"

namespace znn::api {

ZnSymbolResolver* newSymbolResolver(const char* path, void* base) {
  if (!path) return nullptr;
#if defined(__cpp_exceptions)
  try {
#endif
    auto* resolver = new ZnSymbolResolver();
    resolver->image =
        new znn::ElfImage(path, reinterpret_cast<uintptr_t>(base));
    if (!resolver->image->valid()) {
      LOGW("newSymbolResolver %s: invalid", path);
      delete resolver->image;
      delete resolver;
      return nullptr;
    }
    LOGI("newSymbolResolver %s -> %s base=%p", path,
         resolver->image->path().c_str(),
         reinterpret_cast<void*>(resolver->image->base()));
    return resolver;
#if defined(__cpp_exceptions)
  } catch (...) {
    LOGE("newSymbolResolver %s failed with an exception", path);
    return nullptr;
  }
#endif
}

void freeSymbolResolver(ZnSymbolResolver* resolver) {
  if (!resolver) return;
#if defined(__cpp_exceptions)
  try {
#endif
    delete resolver->image;
    delete resolver;
#if defined(__cpp_exceptions)
  } catch (...) {
    LOGE("freeSymbolResolver failed with an exception");
  }
#endif
}

void* getBaseAddress(ZnSymbolResolver* resolver) {
  if (!resolver) return nullptr;
#if defined(__cpp_exceptions)
  try {
#endif
    return reinterpret_cast<void*>(resolver->image->base());
#if defined(__cpp_exceptions)
  } catch (...) {
    LOGE("getBaseAddress failed with an exception");
    return nullptr;
  }
#endif
}

void* symbolLookup(ZnSymbolResolver* resolver, const char* name, bool prefix,
                   size_t* size) {
  if (!resolver || !name) return nullptr;
#if defined(__cpp_exceptions)
  try {
#endif
    if (!prefix) {
      const uintptr_t runtime = resolver->image->runtimeLookup(name, size);
      if (runtime) {
        LOGI("symbolLookup %s -> %p (runtime)", name,
             reinterpret_cast<void*>(runtime));
        return reinterpret_cast<void*>(runtime);
      }
    }

    const znn::SymbolInfo* symbol = resolver->image->lookup(name, prefix);
    if (!symbol) return nullptr;
    if (size) *size = symbol->size;
    LOGI("symbolLookup %s -> %p (elf)", name,
         reinterpret_cast<void*>(symbol->addr));
    return reinterpret_cast<void*>(symbol->addr);
#if defined(__cpp_exceptions)
  } catch (...) {
    LOGE("symbolLookup %s failed with an exception", name);
    return nullptr;
  }
#endif
}

void forEachSymbols(ZnSymbolResolver* resolver,
                    bool (*callback)(const char* name, void* addr, size_t size,
                                     void* data),
                    void* data) {
  if (!resolver || !callback) return;
#if defined(__cpp_exceptions)
  try {
#endif
    resolver->image->forEach(
        [&](const char* name, uintptr_t addr, size_t size) {
          return callback(name, reinterpret_cast<void*>(addr), size, data);
        });
#if defined(__cpp_exceptions)
  } catch (...) {
    LOGE("forEachSymbols failed with an exception");
  }
#endif
}

ZnSymbolResolver* newSymbolResolverUnavailable(const char* path, void*) {
  LOGE(
      "Symbol Resolver API requires module API version >= 2 (newSymbolResolver "
      "%s)",
      path ? path : "(null)");
  return nullptr;
}

void freeSymbolResolverUnavailable(ZnSymbolResolver*) {
  LOGE(
      "Symbol Resolver API requires module API version >= 2 "
      "(freeSymbolResolver)");
}

void* getBaseAddressUnavailable(ZnSymbolResolver*) {
  LOGE("Symbol Resolver API requires module API version >= 2 (getBaseAddress)");
  return nullptr;
}

void* symbolLookupUnavailable(ZnSymbolResolver*, const char* name, bool,
                              size_t*) {
  LOGE("Symbol Resolver API requires module API version >= 2 (symbolLookup %s)",
       name ? name : "(null)");
  return nullptr;
}

void forEachSymbolsUnavailable(ZnSymbolResolver*,
                               bool (*)(const char*, void*, size_t, void*),
                               void*) {
  LOGE("Symbol Resolver API requires module API version >= 2 (forEachSymbols)");
}

}  // namespace znn::api
