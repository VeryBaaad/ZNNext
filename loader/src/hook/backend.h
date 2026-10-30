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

#pragma once

#include <sys/types.h>

#include <cstdint>
#include <string>

namespace znn::hook::backend {

bool dobbyHook(void* target, void* replacement, void** original);
bool dobbyUnhook(void* target);

bool shadowhookInit();
bool shadowhookHook(void* target, void* replacement, void** original);
bool shadowhookUnhook(void* target);

bool rv64hookHook(void* target, void* replacement, void** original);
bool rv64hookUnhook(void* target);

bool and64inlinehookInit();
bool and64inlinehookHook(void* target, void* replacement, void** original);
bool and64inlinehookUnhook(void* target);

bool fridaGumInit();
bool fridaGumHook(void* target, void* replacement, void** original);
bool fridaGumUnhook(void* target);

bool hookzzHook(void* target, void* replacement, void** original);
bool hookzzUnhook(void* target);

bool urhookHook(void* target, void* replacement, void** original);
bool urhookUnhook(void* target);

bool whaleHook(void* target, void* replacement, void** original);
bool whaleUnhook(void* target);

bool lspltHook(dev_t dev, ino_t inode, const char* symbol, void* replacement,
               void** original);

bool bytehookInit();
bool bytehookHook(const std::string& caller_path, const char* symbol,
                  void* replacement, void** original);

bool xhookHook(const std::string& caller_path, const char* symbol,
               void* replacement, void** original);

bool pltiHook(const std::string& caller_path, uintptr_t caller_base,
              const char* symbol, void* replacement, void** original);

bool plthookHook(const std::string& caller_path, uintptr_t caller_base,
                 const char* symbol, void* replacement, void** original);

bool urhookPltHook(const std::string& caller_path, uintptr_t caller_base,
                   const char* symbol, void* replacement, void** original);

}  // namespace znn::hook::backend
