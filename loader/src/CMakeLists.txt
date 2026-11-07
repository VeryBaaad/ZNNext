cmake_minimum_required(VERSION 3.22.1)
project(zn_loader)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_subdirectory(external)

if (LSPLT_STANDALONE)
    find_package(cxx REQUIRED CONFIG)
endif ()

add_library(elfutil STATIC
    utils/elf_util.cpp
    utils/debugdata.cpp
)
target_include_directories(elfutil PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(elfutil PUBLIC lzma xz_embedded)
if (TARGET cxx::cxx)
    target_link_libraries(elfutil PUBLIC cxx::cxx)
endif ()

add_library(zn_loader SHARED
    entry.cpp
    config/hooks.cpp
    config/debugdata.cpp
    process/self.cpp
    utils/dlext.cpp
    hook/engine.cpp
    hook/inline.cpp
    hook/plt.cpp
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
    hook/backend/srx_hook.cpp
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
target_include_directories(zn_loader PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)
if (ANDROID_ABI STREQUAL "arm64-v8a" OR ANDROID_ABI STREQUAL "armeabi-v7a")
    target_include_directories(zn_loader PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/external/shadowhook/src/shadowhook/src/main/cpp/include)
endif ()
if (NOT ANDROID_ABI STREQUAL "riscv64")
    target_include_directories(zn_loader PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/external/whale)
    target_include_directories(zn_loader PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/external/bytehook/bytehook/src/main/cpp/include)
    target_include_directories(zn_loader PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/external/xhook/libxhook/jni)
    target_include_directories(zn_loader PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/external/plthook)
endif ()
if (ANDROID_ABI STREQUAL "arm64-v8a")
    target_include_directories(zn_loader PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/external/urhook/src/include)
endif ()

set(LOADER_LINK_LIBS log elfutil lsplt_static)
if (ANDROID_ABI STREQUAL "riscv64")
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} rv64hook-static)
else()
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} dobby)
endif()
if (ANDROID_ABI STREQUAL "arm64-v8a")
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} and64inlinehook)
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} yukiihook)
endif()
if (ANDROID_ABI STREQUAL "arm64-v8a" OR ANDROID_ABI STREQUAL "armeabi-v7a")
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} shadowhook)
endif()
if (ANDROID_ABI STREQUAL "arm64-v8a" OR ANDROID_ABI STREQUAL "armeabi-v7a" OR ANDROID_ABI STREQUAL "x86_64")
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} hookzz)
endif()
if (NOT ANDROID_ABI STREQUAL "riscv64")
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} whale)
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} bytehook)
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} xhook)
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} plthook)
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} yukiplt)
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} profilo)
endif()
set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} plti)
set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} baaadhook)
if (ANDROID_ABI STREQUAL "arm64-v8a")
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} urhook)
endif()
if (TARGET srx_hook_ffi)
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} srx_hook_ffi)
endif()
if (NOT ANDROID_ABI STREQUAL "riscv64")
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} c++abi unwind)
    target_link_options(zn_loader PRIVATE -Wl,--allow-multiple-definition)
endif()
if (TARGET frida_gum_build)
    set(LOADER_LINK_LIBS ${LOADER_LINK_LIBS} frida-gum)
endif()
target_link_libraries(zn_loader PRIVATE ${LOADER_LINK_LIBS})
if (TARGET frida_gum_build)
    add_dependencies(zn_loader frida_gum_build)
endif()
if (TARGET srx_hook_ffi_archive)
    add_dependencies(zn_loader srx_hook_ffi_archive)
endif()
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
