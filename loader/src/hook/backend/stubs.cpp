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

#include "hook/backend.h"

namespace znn::hook::backend {

#ifndef ZNN_HAS_DOBBY
bool dobbyHook(void*, void*, void**) { return false; }
bool dobbyUnhook(void*) { return false; }
#endif

#ifndef ZNN_HAS_SHADOWHOOK
bool shadowhookInit() { return false; }
bool shadowhookHook(void*, void*, void**) { return false; }
bool shadowhookUnhook(void*) { return false; }
#endif

#ifndef ZNN_HAS_RV64HOOK
bool rv64hookHook(void*, void*, void**) { return false; }
bool rv64hookUnhook(void*) { return false; }
#endif

#ifndef ZNN_HAS_AND64INLINEHOOK
bool and64inlinehookInit() { return false; }
bool and64inlinehookHook(void*, void*, void**) { return false; }
bool and64inlinehookUnhook(void*) { return false; }
#endif

#ifndef ZNN_HAS_FRIDA_GUM
bool fridaGumInit() { return false; }
bool fridaGumHook(void*, void*, void**) { return false; }
bool fridaGumUnhook(void*) { return false; }
#endif

#ifndef ZNN_HAS_HOOKZZ
bool hookzzHook(void*, void*, void**) { return false; }
bool hookzzUnhook(void*) { return false; }
#endif

#ifndef ZNN_HAS_URHOOK
bool urhookHook(void*, void*, void**) { return false; }
bool urhookUnhook(void*) { return false; }
bool urhookPltHook(const std::string&, uintptr_t, const char*, void*, void**) {
  return false;
}
#endif

#ifndef ZNN_HAS_WHALE
bool whaleHook(void*, void*, void**) { return false; }
bool whaleUnhook(void*) { return false; }
#endif

#ifndef ZNN_HAS_BAAADHOOK
bool baaadhookHook(void*, void*, void**) { return false; }
bool baaadhookUnhook(void*) { return false; }
bool baaadhookPltHook(const std::string&, const char*, void*, void**) {
  return false;
}
#endif

#ifndef ZNN_HAS_YUKIIHOOK
bool yukiihookHook(void*, void*, void**) { return false; }
bool yukiihookUnhook(void*) { return false; }
#endif

#ifndef ZNN_HAS_LSPLT
bool lspltHook(dev_t, ino_t, const char*, void*, void**) { return false; }
#endif

#ifndef ZNN_HAS_BYTEHOOK
bool bytehookInit() { return false; }
bool bytehookHook(const std::string&, const char*, void*, void**) {
  return false;
}
#endif

#ifndef ZNN_HAS_XHOOK
bool xhookHook(const std::string&, const char*, void*, void**) { return false; }
#endif

#ifndef ZNN_HAS_PLTI
bool pltiHook(const std::string&, uintptr_t, const char*, void*, void**) {
  return false;
}
#endif

#ifndef ZNN_HAS_PLTHOOK
bool plthookHook(const std::string&, uintptr_t, const char*, void*, void**) {
  return false;
}
#endif

#ifndef ZNN_HAS_YUKIPLT
bool yukipltHook(const std::string&, uintptr_t, const char*, void*, void**) {
  return false;
}
#endif

#ifndef ZNN_HAS_PROFILO
bool profiloInit() { return false; }
bool profiloHook(const std::string&, uintptr_t, const char*, void*, void**) {
  return false;
}
#endif

#ifndef ZNN_HAS_SRXHOOK
bool srxhookInit() { return false; }
bool srxhookPltHook(const std::string&, uintptr_t, const char*, void*, void**) {
  return false;
}
#endif

}  // namespace znn::hook::backend
