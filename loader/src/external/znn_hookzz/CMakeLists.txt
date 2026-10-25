cmake_minimum_required(VERSION 3.22.1)

if (NOT ANDROID_ABI STREQUAL "arm64-v8a"
        AND NOT ANDROID_ABI STREQUAL "armeabi-v7a"
        AND NOT ANDROID_ABI STREQUAL "x86_64")
    return()
endif ()

set(HOOKZZ_SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../hookzz")

if (ANDROID_ABI STREQUAL "arm64-v8a")
    set(HOOKZZ_ARCH "arm64")
elseif (ANDROID_ABI STREQUAL "armeabi-v7a")
    set(HOOKZZ_ARCH "arm")
else ()
    set(HOOKZZ_ARCH "x64")
endif ()

file(GLOB HOOKZZ_SOURCES
    "${HOOKZZ_SOURCE_DIR}/srcxx/*.cpp"
    "${HOOKZZ_SOURCE_DIR}/srcxx/core/arch/*.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/core/arch/${HOOKZZ_ARCH}/*.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/core/modules/assembler/assembler.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/core/modules/assembler/assembler-${HOOKZZ_ARCH}.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/core/modules/codegen/codegen-${HOOKZZ_ARCH}.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/ExecMemory/*.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/ExecMemory/CodeBuffer/CodeBufferBase.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/ExecMemory/CodeBuffer/code-buffer-${HOOKZZ_ARCH}.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/InstructionRelocation/${HOOKZZ_ARCH}/*.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/InterceptRouting/*.cpp"
    "${HOOKZZ_SOURCE_DIR}/srcxx/InterceptRoutingTrampoline/${HOOKZZ_ARCH}/*.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/InterceptRoutingPlugin/FunctionInlineReplace/*.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/UserMode/ExecMemory/code-patch-tool-posix.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/UserMode/PlatformInterface/Common/platform-posix.cc"
    "${HOOKZZ_SOURCE_DIR}/srcxx/UserMode/PlatformInterface/Common/platform-posix/*.cc"
    "${HOOKZZ_SOURCE_DIR}/OneLib/logging/*.c"
    "${HOOKZZ_SOURCE_DIR}/OneLib/logging/*.cc"
    "${HOOKZZ_SOURCE_DIR}/OneLib/stdcxx/*.cc"
)

if (HOOKZZ_ARCH STREQUAL "x64")
    file(GLOB HOOKZZ_X86_SOURCES
        "${HOOKZZ_SOURCE_DIR}/srcxx/InstructionRelocation/x86/*.cc"
    )
    list(APPEND HOOKZZ_SOURCES ${HOOKZZ_X86_SOURCES})
endif ()

list(APPEND HOOKZZ_SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/clear-cache-tool.cc")

add_library(hookzz STATIC ${HOOKZZ_SOURCES})
set_target_properties(hookzz PROPERTIES
    CXX_STANDARD 11
    CXX_STANDARD_REQUIRED ON
    CXX_EXTENSIONS ON
)
target_include_directories(hookzz
    PUBLIC "${HOOKZZ_SOURCE_DIR}/include"
    PRIVATE
        "${HOOKZZ_SOURCE_DIR}"
        "${HOOKZZ_SOURCE_DIR}/srcxx"
        "${HOOKZZ_SOURCE_DIR}/srcxx/UserMode"
        "${HOOKZZ_SOURCE_DIR}/OneLib/logging"
        "${HOOKZZ_SOURCE_DIR}/OneLib/stdcxx"
        "${HOOKZZ_SOURCE_DIR}/OneLib/include"
)
target_compile_options(hookzz PRIVATE
    -w
    -fPIC
    -include "${CMAKE_CURRENT_SOURCE_DIR}/hookzz-symbols.h"
)
