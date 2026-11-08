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

#include "hook/engine.h"

#include <string.h>
#include <unistd.h>

#include "hook/backend.h"
#include "log.h"

namespace znn::hook {
namespace {

InlineEngine g_inline_engine = defaultInlineEngine();
PltEngine g_plt_engine = defaultPltEngine();

}  // namespace

const char* inlineEngineName(InlineEngine e) {
  switch (e) {
    case InlineEngine::kDobby:
      return "dobby";
    case InlineEngine::kShadowHook:
      return "shadowhook";
    case InlineEngine::kAnd64InlineHook:
      return "and64inlinehook";
    case InlineEngine::kFridaGum:
      return "frida-gum";
    case InlineEngine::kRv64Hook:
      return "rv64hook";
    case InlineEngine::kHookZz:
      return "hookzz";
    case InlineEngine::kUrHook:
      return "urhook";
    case InlineEngine::kWhale:
      return "whale";
    case InlineEngine::kBaaadHook:
      return "baaadhook";
    case InlineEngine::kYukiIhook:
      return "yukiihook";
    case InlineEngine::kDisable:
      return "disable";
  }
  return "unknown";
}

const char* pltEngineName(PltEngine e) {
  switch (e) {
    case PltEngine::kLsplt:
      return "lsplt";
    case PltEngine::kByteHook:
      return "bytehook";
    case PltEngine::kXHook:
      return "xhook";
    case PltEngine::kPlti:
      return "plti";
    case PltEngine::kPltHook:
      return "plthook";
    case PltEngine::kUrHook:
      return "urhook";
    case PltEngine::kBaaadHook:
      return "baaadhook";
    case PltEngine::kYukiPlt:
      return "yukiplt";
    case PltEngine::kProfilo:
      return "profilo";
    case PltEngine::kSrxHook:
      return "srxhook";
    case PltEngine::kDisable:
      return "disable";
  }
  return "unknown";
}

bool inlineEngineSupported(InlineEngine e) {
  switch (e) {
    case InlineEngine::kDobby:
#ifdef ZNN_HAS_DOBBY
      return true;
#else
      return false;
#endif
    case InlineEngine::kShadowHook:
#ifdef ZNN_HAS_SHADOWHOOK
      return true;
#else
      return false;
#endif
    case InlineEngine::kAnd64InlineHook:
#ifdef ZNN_HAS_AND64INLINEHOOK
      return true;
#else
      return false;
#endif
    case InlineEngine::kFridaGum:
#ifdef ZNN_HAS_FRIDA_GUM
      return true;
#else
      return false;
#endif
    case InlineEngine::kRv64Hook:
#ifdef ZNN_HAS_RV64HOOK
      return true;
#else
      return false;
#endif
    case InlineEngine::kHookZz:
#ifdef ZNN_HAS_HOOKZZ
      return true;
#else
      return false;
#endif
    case InlineEngine::kUrHook:
#ifdef ZNN_HAS_URHOOK
      return true;
#else
      return false;
#endif
    case InlineEngine::kWhale:
#ifdef ZNN_HAS_WHALE
      return true;
#else
      return false;
#endif
    case InlineEngine::kBaaadHook:
#ifdef ZNN_HAS_BAAADHOOK
      return true;
#else
      return false;
#endif
    case InlineEngine::kYukiIhook:
#ifdef ZNN_HAS_YUKIIHOOK
      return true;
#else
      return false;
#endif
    case InlineEngine::kDisable:
      return true;
  }
  return false;
}

bool pltEngineSupported(PltEngine e) {
  switch (e) {
    case PltEngine::kLsplt:
#ifdef ZNN_HAS_LSPLT
      return true;
#else
      return false;
#endif
    case PltEngine::kByteHook:
#ifdef ZNN_HAS_BYTEHOOK
      return true;
#else
      return false;
#endif
    case PltEngine::kXHook:
#ifdef ZNN_HAS_XHOOK
      return true;
#else
      return false;
#endif
    case PltEngine::kPlti:
#ifdef ZNN_HAS_PLTI
      return true;
#else
      return false;
#endif
    case PltEngine::kPltHook:
#ifdef ZNN_HAS_PLTHOOK
      return true;
#else
      return false;
#endif
    case PltEngine::kUrHook:
#ifdef ZNN_HAS_URHOOK
      return true;
#else
      return false;
#endif
    case PltEngine::kBaaadHook:
#ifdef ZNN_HAS_BAAADHOOK
      return true;
#else
      return false;
#endif
    case PltEngine::kYukiPlt:
#ifdef ZNN_HAS_YUKIPLT
      return true;
#else
      return false;
#endif
    case PltEngine::kProfilo:
#ifdef ZNN_HAS_PROFILO
      return true;
#else
      return false;
#endif
    case PltEngine::kSrxHook:
#ifdef ZNN_HAS_SRXHOOK
      return true;
#else
      return false;
#endif
    case PltEngine::kDisable:
      return true;
  }
  return false;
}

InlineEngine defaultInlineEngine() {
#ifdef ZNN_HAS_RV64HOOK
  return InlineEngine::kRv64Hook;
#elif defined(ZNN_HAS_DOBBY)
  return InlineEngine::kDobby;
#elif defined(ZNN_HAS_FRIDA_GUM)
  return InlineEngine::kFridaGum;
#else
  return InlineEngine::kDisable;
#endif
}

PltEngine defaultPltEngine() {
#ifdef ZNN_HAS_LSPLT
  return PltEngine::kLsplt;
#elif defined(ZNN_HAS_XHOOK)
  return PltEngine::kXHook;
#else
  return PltEngine::kDisable;
#endif
}

bool parseInlineEngine(const char* name, InlineEngine* out) {
  if (!name) return false;
  InlineEngine e;
  if (strcmp(name, "dobby") == 0) {
    e = InlineEngine::kDobby;
  } else if (strcmp(name, "shadowhook") == 0) {
    e = InlineEngine::kShadowHook;
  } else if (strcmp(name, "and64inlinehook") == 0) {
    e = InlineEngine::kAnd64InlineHook;
  } else if (strcmp(name, "frida-gum") == 0) {
    e = InlineEngine::kFridaGum;
  } else if (strcmp(name, "rv64hook") == 0) {
    e = InlineEngine::kRv64Hook;
  } else if (strcmp(name, "hookzz") == 0) {
    e = InlineEngine::kHookZz;
  } else if (strcmp(name, "urhook") == 0) {
    e = InlineEngine::kUrHook;
  } else if (strcmp(name, "whale") == 0) {
    e = InlineEngine::kWhale;
  } else if (strcmp(name, "baaadhook") == 0) {
    e = InlineEngine::kBaaadHook;
  } else if (strcmp(name, "yukiihook") == 0) {
    e = InlineEngine::kYukiIhook;
  } else if (strcmp(name, "disable") == 0) {
    e = InlineEngine::kDisable;
  } else {
    return false;
  }
  if (!inlineEngineSupported(e)) return false;
  *out = e;
  return true;
}

bool parsePltEngine(const char* name, PltEngine* out) {
  if (!name) return false;
  PltEngine e;
  if (strcmp(name, "lsplt") == 0) {
    e = PltEngine::kLsplt;
  } else if (strcmp(name, "bytehook") == 0) {
    e = PltEngine::kByteHook;
  } else if (strcmp(name, "xhook") == 0) {
    e = PltEngine::kXHook;
  } else if (strcmp(name, "plti") == 0) {
    e = PltEngine::kPlti;
  } else if (strcmp(name, "plthook") == 0) {
    e = PltEngine::kPltHook;
  } else if (strcmp(name, "urhook") == 0) {
    e = PltEngine::kUrHook;
  } else if (strcmp(name, "baaadhook") == 0) {
    e = PltEngine::kBaaadHook;
  } else if (strcmp(name, "yukiplt") == 0) {
    e = PltEngine::kYukiPlt;
  } else if (strcmp(name, "profilo") == 0) {
    e = PltEngine::kProfilo;
  } else if (strcmp(name, "srxhook") == 0) {
    e = PltEngine::kSrxHook;
  } else if (strcmp(name, "disable") == 0) {
    e = PltEngine::kDisable;
  } else {
    return false;
  }
  if (!pltEngineSupported(e)) return false;
  *out = e;
  return true;
}

InlineEngine inlineEngine() { return g_inline_engine; }

PltEngine pltEngine() { return g_plt_engine; }

void setInlineEngine(InlineEngine e) {
  if (inlineEngineSupported(e)) g_inline_engine = e;
}

void setPltEngine(PltEngine e) {
  if (pltEngineSupported(e)) g_plt_engine = e;
}

void ensureInlineEngineReady() {
  bool ready = true;
  switch (g_inline_engine) {
    case InlineEngine::kShadowHook:
      ready = backend::shadowhookInit();
      break;
    case InlineEngine::kAnd64InlineHook:
      ready = backend::and64inlinehookInit();
      break;
    case InlineEngine::kFridaGum:
      ready = backend::fridaGumInit();
      break;
    default:
      return;
  }
  if (ready) return;

  const InlineEngine fallback = defaultInlineEngine();
  LOGE("%s unavailable in pid %d, falling back to inline=%s for this process",
       inlineEngineName(g_inline_engine), getpid(), inlineEngineName(fallback));
  g_inline_engine = fallback;
}

void ensurePltEngineReady() {
  bool ready = true;
  switch (g_plt_engine) {
    case PltEngine::kByteHook:
      ready = backend::bytehookInit();
      break;
    case PltEngine::kProfilo:
      ready = backend::profiloInit();
      break;
    case PltEngine::kSrxHook:
      ready = backend::srxhookInit();
      break;
    default:
      return;
  }
  if (ready) return;

  const PltEngine fallback = defaultPltEngine();
  LOGE("%s unavailable in pid %d, falling back to plt=%s for this process",
       pltEngineName(g_plt_engine), getpid(), pltEngineName(fallback));
  g_plt_engine = fallback;
}

}  // namespace znn::hook
