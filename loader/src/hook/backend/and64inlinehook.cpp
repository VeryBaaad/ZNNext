/*
 * This file is part of Zygisk Next Next.
 *
 * Zygisk Next Next is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Zygisk Next Next is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Zygisk Next Next. If not, see <https://www.gnu.org/licenses/>.
 *
 * Copyright (C) 2026 VeryBaaad <verybaaad@outlook.com>
 */

#include "hook/backend.h"

#include "log.h"

#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include <map>
#include <mutex>

#if defined(__aarch64__)
#include <And64InlineHook.hpp>
#endif

namespace znn::hook::backend {

#if defined(__aarch64__)

namespace {

constexpr size_t kPatchWords = 5;
constexpr size_t kPatchBytes = kPatchWords * sizeof(uint32_t);
constexpr size_t kTrampolineBytes = 4096;

struct HookRecord {
    uint32_t backup[kPatchWords];
    void* trampoline;
};

std::mutex g_mutex;
std::map<uintptr_t, HookRecord> g_records;

bool makePatchWritable(void* target) {
    const long page_size = sysconf(_SC_PAGESIZE);
    if (page_size <= 0) return false;

    const uintptr_t align_mask = static_cast<uintptr_t>(page_size) - 1;
    const uintptr_t begin = reinterpret_cast<uintptr_t>(target) & ~align_mask;
    const uintptr_t end =
        (reinterpret_cast<uintptr_t>(target) + kPatchBytes + align_mask) & ~align_mask;
    return mprotect(reinterpret_cast<void*>(begin), end - begin,
                    PROT_READ | PROT_WRITE | PROT_EXEC) == 0;
}

void flushPatch(void* target) {
    __builtin___clear_cache(reinterpret_cast<char*>(target),
                            reinterpret_cast<char*>(target) + kPatchBytes);
}

}  //namespace

bool and64inlinehookHook(void* target, void* replacement, void** original) {
    std::lock_guard<std::mutex> lk(g_mutex);
    const uintptr_t key = reinterpret_cast<uintptr_t>(target);

    if (g_records.find(key) != g_records.end()) {
        LOGE("inlineHook %p failed: already hooked", target);
        return false;
    }

    void* trampoline = mmap(nullptr, kTrampolineBytes, PROT_READ | PROT_WRITE | PROT_EXEC,
                            MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (trampoline == MAP_FAILED) {
        LOGE("inlineHook %p failed: no trampoline memory", target);
        return false;
    }
    if (!makePatchWritable(target)) {
        munmap(trampoline, kTrampolineBytes);
        LOGE("inlineHook %p failed: the target is not writable", target);
        return false;
    }

    HookRecord record{};
    memcpy(record.backup, target, kPatchBytes);
    record.trampoline = trampoline;

    if (A64HookFunctionV(target, replacement, trampoline, kTrampolineBytes) == nullptr) {
        memcpy(target, record.backup, kPatchBytes);
        flushPatch(target);
        munmap(trampoline, kTrampolineBytes);
        LOGE("inlineHook %p failed", target);
        return false;
    }

    g_records.emplace(key, record);
    if (original) *original = trampoline;
    return true;
}

bool and64inlinehookUnhook(void* target) {
    std::lock_guard<std::mutex> lk(g_mutex);
    const uintptr_t key = reinterpret_cast<uintptr_t>(target);

    const auto it = g_records.find(key);
    if (it == g_records.end()) {
        LOGE("inlineUnhook %p failed: not hooked", target);
        return false;
    }
    if (!makePatchWritable(target)) {
        LOGE("inlineUnhook %p failed: the target is not writable", target);
        return false;
    }

    memcpy(target, it->second.backup, kPatchBytes);
    flushPatch(target);
    munmap(it->second.trampoline, kTrampolineBytes);
    g_records.erase(it);
    return true;
}

#else

bool and64inlinehookHook(void* target, void* replacement, void** original) {
    (void)target;
    (void)replacement;
    (void)original;
    return false;
}

bool and64inlinehookUnhook(void* target) {
    (void)target;
    return false;
}

#endif

}  //namespace znn::hook::backend
