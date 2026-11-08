cmake_minimum_required(VERSION 3.22.1)
project(zn_loader)

set(ZNN_LEGACY_ABIS armeabi mips mips64)
if (ANDROID_ABI IN_LIST ZNN_LEGACY_ABIS)
    set(ZNN_LEGACY ON)
    set(CMAKE_CXX_STANDARD 14)
else ()
    set(ZNN_LEGACY OFF)
    set(CMAKE_CXX_STANDARD 20)
endif ()
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_subdirectory(external)

if (LSPLT_STANDALONE)
    find_package(cxx REQUIRED CONFIG)
endif ()

set(ZNN_INLINE_BACKENDS)
set(ZNN_PLT_BACKENDS)
if (ANDROID_ABI STREQUAL "arm64-v8a")
    set(ZNN_INLINE_BACKENDS
        DOBBY SHADOWHOOK AND64INLINEHOOK FRIDA_GUM HOOKZZ URHOOK WHALE BAAADHOOK YUKIIHOOK)
    set(ZNN_PLT_BACKENDS
        LSPLT PLTI BYTEHOOK XHOOK PLTHOOK URHOOK BAAADHOOK YUKIPLT PROFILO SRXHOOK)
elseif (ANDROID_ABI STREQUAL "armeabi-v7a")
    set(ZNN_INLINE_BACKENDS DOBBY SHADOWHOOK FRIDA_GUM HOOKZZ WHALE BAAADHOOK)
    set(ZNN_PLT_BACKENDS LSPLT PLTI BYTEHOOK XHOOK PLTHOOK BAAADHOOK YUKIPLT PROFILO)
elseif (ANDROID_ABI STREQUAL "x86")
    set(ZNN_INLINE_BACKENDS DOBBY FRIDA_GUM WHALE BAAADHOOK)
    set(ZNN_PLT_BACKENDS LSPLT PLTI BYTEHOOK XHOOK PLTHOOK BAAADHOOK YUKIPLT PROFILO)
elseif (ANDROID_ABI STREQUAL "x86_64")
    set(ZNN_INLINE_BACKENDS DOBBY FRIDA_GUM HOOKZZ WHALE BAAADHOOK)
    set(ZNN_PLT_BACKENDS LSPLT PLTI BYTEHOOK XHOOK PLTHOOK BAAADHOOK YUKIPLT PROFILO SRXHOOK)
elseif (ANDROID_ABI STREQUAL "riscv64")
    set(ZNN_INLINE_BACKENDS RV64HOOK BAAADHOOK)
    set(ZNN_PLT_BACKENDS LSPLT PLTI BAAADHOOK)
elseif (ANDROID_ABI STREQUAL "armeabi")
    set(ZNN_INLINE_BACKENDS FRIDA_GUM)
    set(ZNN_PLT_BACKENDS XHOOK)
elseif (ANDROID_ABI STREQUAL "mips" OR ANDROID_ABI STREQUAL "mips64")
    set(ZNN_INLINE_BACKENDS)
    set(ZNN_PLT_BACKENDS)
else ()
    message(FATAL_ERROR "unsupported Android ABI: ${ANDROID_ABI}")
endif ()

foreach (backend IN LISTS ZNN_INLINE_BACKENDS ZNN_PLT_BACKENDS)
    set(ZNN_HAS_${backend} TRUE)
endforeach ()

add_library(elfutil STATIC
    utils/elf_util.cpp
    utils/debugdata.cpp
    utils/debugdata_archive.cpp
)
target_include_directories(elfutil PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(elfutil PUBLIC archive_static lzma xz_embedded)
if (TARGET cxx::cxx)
    target_link_libraries(elfutil PUBLIC cxx::cxx)
endif ()

set(ZNN_LOADER_SOURCES
    entry.cpp
    config/hooks.cpp
    config/debugdata.cpp
    process/self.cpp
    utils/dlext.cpp
    hook/engine.cpp
    hook/inline.cpp
    hook/plt.cpp
    api/hook.cpp
    api/symbols.cpp
    api/companion.cpp
    api/runtime.cpp
    api/table.cpp
    hyos/state.cpp
    hyos/hooks.cpp
    hyos/runtime.cpp
    ipc/channel.cpp
    module/plan.cpp
    module/loader.cpp
)

if (ZNN_LEGACY)
    list(APPEND ZNN_LOADER_SOURCES
        utils/legacy_compat.cpp
        hook/backend/stubs.cpp)
    if (ZNN_HAS_FRIDA_GUM)
        list(APPEND ZNN_LOADER_SOURCES hook/backend/frida-gum.cpp)
    endif ()
    if (ANDROID_ABI STREQUAL "armeabi")
        list(APPEND ZNN_LOADER_SOURCES hook/backend/xhook.cpp)
    endif ()
else ()
    list(APPEND ZNN_LOADER_SOURCES
        hook/backend/dobby.cpp
        hook/backend/shadowhook.cpp
        hook/backend/rv64hook.cpp
        hook/backend/and64inlinehook.cpp
        hook/backend/frida-gum.cpp
        hook/backend/hookzz.cpp
        hook/backend/whale.cpp
        hook/backend/lsplt.cpp
        hook/backend/bytehook.cpp
        hook/backend/xhook.cpp
        hook/backend/plti.cpp
        hook/backend/plthook.cpp
        hook/backend/yukiplt.cpp
        hook/backend/profilo.cpp
        hook/backend/urhook.cpp
        hook/backend/baaadhook.cpp
        hook/backend/yukiihook.cpp
        hook/backend/srx_hook.cpp)
endif ()

add_library(zn_loader SHARED ${ZNN_LOADER_SOURCES})
foreach (backend IN LISTS ZNN_INLINE_BACKENDS ZNN_PLT_BACKENDS)
    target_compile_definitions(zn_loader PRIVATE ZNN_HAS_${backend}=1)
endforeach ()

target_include_directories(zn_loader PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)
if (ZNN_HAS_SHADOWHOOK)
    target_include_directories(zn_loader PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/external/shadowhook/src/shadowhook/src/main/cpp/include)
endif ()
if (ZNN_HAS_WHALE)
    target_include_directories(zn_loader PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/external/whale)
endif ()
if (ZNN_HAS_BYTEHOOK)
    target_include_directories(zn_loader PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/external/bytehook/bytehook/src/main/cpp/include)
endif ()
if (ZNN_HAS_XHOOK)
    target_include_directories(zn_loader PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/external/xhook/libxhook/jni)
endif ()
if (ZNN_HAS_PLTHOOK)
    target_include_directories(zn_loader PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/external/plthook)
endif ()
if (ZNN_HAS_URHOOK)
    target_include_directories(zn_loader PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/external/urhook/src/include)
endif ()

set(LOADER_LINK_LIBS log elfutil)
if (ZNN_LEGACY)
    if (TARGET frida_gum_build)
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} frida-gum)
    endif ()
    if (ZNN_HAS_XHOOK)
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} xhook)
    endif ()
else ()
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} lsplt_static)
    if (ANDROID_ABI STREQUAL "riscv64")
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} rv64hook-static)
    else ()
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} dobby)
    endif ()
    if (ANDROID_ABI STREQUAL "arm64-v8a")
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} and64inlinehook)
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} yukiihook)
    endif ()
    if (ANDROID_ABI STREQUAL "arm64-v8a" OR ANDROID_ABI STREQUAL "armeabi-v7a")
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} shadowhook)
    endif ()
    if (ANDROID_ABI STREQUAL "arm64-v8a" OR ANDROID_ABI STREQUAL "armeabi-v7a" OR ANDROID_ABI STREQUAL "x86_64")
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} hookzz)
    endif ()
    if (NOT ANDROID_ABI STREQUAL "riscv64")
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} whale)
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} bytehook)
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} xhook)
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} plthook)
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} yukiplt)
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} profilo)
    endif ()
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} plti)
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} baaadhook)
    if (ANDROID_ABI STREQUAL "arm64-v8a")
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} urhook)
    endif ()
    if (TARGET srx_hook_ffi)
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} srx_hook_ffi)
    endif ()
    if (NOT ANDROID_ABI STREQUAL "riscv64")
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} c++abi unwind)
        target_link_options(zn_loader PRIVATE -Wl,--allow-multiple-definition)
    endif ()
    if (TARGET frida_gum_build)
        set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} frida-gum)
    endif ()
endif ()

target_link_libraries(zn_loader PRIVATE ${LOADER_LINK_LIBS})
if (TARGET frida_gum_build)
    add_dependencies(zn_loader frida_gum_build)
endif ()
if (TARGET srx_hook_ffi_archive)
    add_dependencies(zn_loader srx_hook_ffi_archive)
endif ()
if (TARGET cxx::cxx)
    target_link_libraries(zn_loader PRIVATE cxx::cxx)
endif ()

target_link_options(zn_loader PRIVATE
    -Wl,-z,max-page-size=16384
    -Wl,-z,common-page-size=16384)
target_compile_options(zn_loader PRIVATE -Wall -Wextra -fvisibility=hidden)
if (NOT ANDROID_ABI STREQUAL "riscv64")
    target_compile_options(zn_loader PRIVATE -fexceptions -funwind-tables)
    target_compile_options(elfutil PRIVATE -fexceptions -funwind-tables)
endif ()
set_source_files_properties(hook/backend/profilo.cpp PROPERTIES
    COMPILE_OPTIONS "-fexceptions;-funwind-tables")
