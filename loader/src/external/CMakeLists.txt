cmake_minimum_required(VERSION 3.22.1)
project(external)

if (LSPLT_STANDALONE)
    find_package(cxx REQUIRED CONFIG)
endif ()

if (ANDROID_ABI STREQUAL "riscv64")
	set(ANDROID_PLATFORM "android-35")
endif ()

option(LSPLT_BUILD_SHARED "Build LSPlt as a shared library" OFF)
if (NOT ZNN_LEGACY)
    add_subdirectory(lsplt/lsplt/src/main/jni)
endif ()

if (NOT ZNN_LEGACY AND (ANDROID_ABI STREQUAL "arm64-v8a" OR ANDROID_ABI STREQUAL "armeabi-v7a"))
    enable_language(ASM)
    set(SHADOWHOOK_SRC ${CMAKE_CURRENT_SOURCE_DIR}/shadowhook/src/shadowhook/src/main/cpp)
    if (ANDROID_ABI STREQUAL "arm64-v8a")
        set(SHADOWHOOK_ARCH "arm64")
    else ()
        set(SHADOWHOOK_ARCH "arm")
    endif ()
    file(GLOB SHADOWHOOK_SOURCES
        ${SHADOWHOOK_SRC}/*.c
        ${SHADOWHOOK_SRC}/arch/${SHADOWHOOK_ARCH}/*.c
        ${SHADOWHOOK_SRC}/arch/${SHADOWHOOK_ARCH}/*.S
        ${SHADOWHOOK_SRC}/common/*.c
        ${SHADOWHOOK_SRC}/third_party/*/*.c
    )
    # DO NOT TOUCH THIS.
    list(REMOVE_ITEM SHADOWHOOK_SOURCES ${SHADOWHOOK_SRC}/sh_linker.c)
    list(APPEND SHADOWHOOK_SOURCES
        ${CMAKE_CURRENT_SOURCE_DIR}/shadowhook/sh_linker.c)
    add_library(shadowhook STATIC ${SHADOWHOOK_SOURCES})
    target_include_directories(shadowhook PRIVATE
        ${SHADOWHOOK_SRC}
        ${SHADOWHOOK_SRC}/include
        ${SHADOWHOOK_SRC}/arch/${SHADOWHOOK_ARCH}
        ${SHADOWHOOK_SRC}/common
        ${SHADOWHOOK_SRC}/third_party/xdl
        ${SHADOWHOOK_SRC}/third_party/bsd
        ${SHADOWHOOK_SRC}/third_party/lss
    )
    target_compile_options(shadowhook PRIVATE -w)
endif ()

if (ANDROID_ABI STREQUAL "arm64-v8a" OR ANDROID_ABI STREQUAL "armeabi-v7a" OR ANDROID_ABI STREQUAL "armeabi")
    enable_language(ASM)
    set(SANDHOOK_SRC ${CMAKE_CURRENT_SOURCE_DIR}/sandhook/nativehook/src/main/cpp)
    set(SANDHOOK_SOURCES
        ${SANDHOOK_SRC}/sandhook_native.cpp
        ${SANDHOOK_SRC}/decoder/decoder.cpp
        ${SANDHOOK_SRC}/relocate/code_relocate.cpp
        ${SANDHOOK_SRC}/elf/elf.cpp
        ${SANDHOOK_SRC}/assembler/assembler.cpp
        ${SANDHOOK_SRC}/buffer/code_buffer.cpp
        ${SANDHOOK_SRC}/utils/platform.cpp
        ${SANDHOOK_SRC}/hook/hook.cpp
    )
    if (ANDROID_ABI STREQUAL "arm64-v8a")
        set(SANDHOOK_ARCH "arm64")
        list(APPEND SANDHOOK_SOURCES
            ${SANDHOOK_SRC}/archs/arm/arm64/assembler/assembler_arm64.cpp
            ${SANDHOOK_SRC}/archs/arm/arm64/inst/inst_arm64.cpp
            ${SANDHOOK_SRC}/archs/arm/arm64/register/register_arm64.cpp
            ${SANDHOOK_SRC}/archs/arm/arm64/register/register_list_arm64.cpp
            ${SANDHOOK_SRC}/archs/arm/arm64/decoder/decoder_arm64.cpp
            ${SANDHOOK_SRC}/archs/arm/arm64/relocate/code_relocate_arm64.cpp
            ${SANDHOOK_SRC}/archs/arm/arm64/hook/hook_arm64.cpp
        )
    else ()
        set(SANDHOOK_ARCH "arm32")
        list(APPEND SANDHOOK_SOURCES
            ${SANDHOOK_SRC}/archs/arm/arm32/inst/inst_arm32.cpp
            ${SANDHOOK_SRC}/archs/arm/arm32/inst/inst_t32.cpp
            ${SANDHOOK_SRC}/archs/arm/arm32/inst/inst_t16.cpp
            ${SANDHOOK_SRC}/archs/arm/arm32/register/register_arm32.cpp
            ${SANDHOOK_SRC}/archs/arm/arm32/register/register_list_arm32.cpp
            ${SANDHOOK_SRC}/archs/arm/arm32/assembler/assembler_arm32.cpp
            ${SANDHOOK_SRC}/archs/arm/arm32/decoder/decoder_arm32.cpp
            ${SANDHOOK_SRC}/archs/arm/arm32/hook/hook_arm32.cpp
            ${SANDHOOK_SRC}/archs/arm/arm32/hook/breakpoint_shellcode.S
            ${SANDHOOK_SRC}/archs/arm/arm32/relocate/code_relocate_arm32.cpp
        )
    endif ()
    add_library(sandhook STATIC ${SANDHOOK_SOURCES})
    set_target_properties(sandhook PROPERTIES
        CXX_STANDARD 14
        CXX_STANDARD_REQUIRED ON
        CXX_EXTENSIONS ON
        POSITION_INDEPENDENT_CODE ON
    )
    target_compile_definitions(sandhook PRIVATE
        "PAGE_SIZE=sysconf(_SC_PAGESIZE)")
    target_compile_options(sandhook PRIVATE
        -w
        "$<$<COMPILE_LANGUAGE:CXX>:-fexceptions>"
        "$<$<COMPILE_LANGUAGE:CXX>:-funwind-tables>"
        "$<$<COMPILE_LANGUAGE:CXX>:-fasynchronous-unwind-tables>"
        "$<$<COMPILE_LANGUAGE:CXX>:-include>"
        "$<$<COMPILE_LANGUAGE:CXX>:${CMAKE_CURRENT_SOURCE_DIR}/sandhook_compat.h>")
    target_include_directories(sandhook
        PUBLIC ${SANDHOOK_SRC}
        PRIVATE
            ${SANDHOOK_SRC}/asm
            ${SANDHOOK_SRC}/decoder
            ${SANDHOOK_SRC}/elf
            ${SANDHOOK_SRC}/utils
            ${SANDHOOK_SRC}/includes
            ${SANDHOOK_SRC}/buffer
            ${SANDHOOK_SRC}/archs/arm
            ${SANDHOOK_SRC}/archs/arm/${SANDHOOK_ARCH}/inst
            ${SANDHOOK_SRC}/archs/arm/${SANDHOOK_ARCH}/register
            ${SANDHOOK_SRC}/archs/arm/${SANDHOOK_ARCH}/assembler
            ${SANDHOOK_SRC}/archs/arm/${SANDHOOK_ARCH}/decoder
            ${SANDHOOK_SRC}/archs/arm/${SANDHOOK_ARCH}/relocate
            ${SANDHOOK_SRC}/archs/arm/${SANDHOOK_ARCH}/hook
    )
    if (TARGET cxx::cxx)
        get_target_property(SANDHOOK_CXX_INCLUDES cxx::cxx INTERFACE_INCLUDE_DIRECTORIES)
        if (SANDHOOK_CXX_INCLUDES)
            target_include_directories(sandhook SYSTEM PRIVATE ${SANDHOOK_CXX_INCLUDES})
        endif ()
    endif ()
endif ()

if (NOT ZNN_LEGACY AND NOT ANDROID_ABI STREQUAL "riscv64")
    set(BYTEHOOK_SRC ${CMAKE_CURRENT_SOURCE_DIR}/bytehook/bytehook/src/main/cpp)
    file(GLOB BYTEHOOK_SOURCES ${BYTEHOOK_SRC}/*.c)
    add_library(bytehook STATIC ${BYTEHOOK_SOURCES})
    target_compile_definitions(bytehook PRIVATE
        bytesig_init=bytehook_bytesig_init
        bytesig_protect=bytehook_bytesig_protect
        bytesig_unprotect=bytehook_bytesig_unprotect
    )
    target_include_directories(bytehook PRIVATE
        ${BYTEHOOK_SRC}
        ${BYTEHOOK_SRC}/include
        ${BYTEHOOK_SRC}/third_party/bsd
        ${BYTEHOOK_SRC}/third_party/lss
    )
    if (ANDROID_ABI STREQUAL "arm64-v8a" OR ANDROID_ABI STREQUAL "armeabi-v7a")
        target_include_directories(bytehook PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/shadowhook/src/shadowhook/src/main/cpp/include)
        target_link_libraries(bytehook PRIVATE shadowhook)
    endif ()
    target_compile_options(bytehook PRIVATE -w)

    add_library(plthook STATIC
        ${CMAKE_CURRENT_SOURCE_DIR}/plthook/plthook_elf.c
        ${CMAKE_CURRENT_SOURCE_DIR}/plthook/plthook.h
    )
    target_compile_options(plthook PRIVATE -w)
endif ()

if (ANDROID_ABI STREQUAL "armeabi" OR (NOT ZNN_LEGACY AND NOT ANDROID_ABI STREQUAL "riscv64"))
    set(XHOOK_SRC ${CMAKE_CURRENT_SOURCE_DIR}/xhook/libxhook/jni)
    file(GLOB XHOOK_SOURCES ${XHOOK_SRC}/*.c)
    add_library(xhook STATIC ${XHOOK_SOURCES})
    target_compile_definitions(xhook PRIVATE
        "PAGE_SIZE=sysconf(_SC_PAGESIZE)"
        "PAGE_MASK=(~(sysconf(_SC_PAGESIZE)-1))"
    )
    target_compile_options(xhook PRIVATE -w)
endif ()

if (ANDROID_ABI STREQUAL "arm64-v8a")
    add_library(and64inlinehook STATIC
        ${CMAKE_CURRENT_SOURCE_DIR}/and64inlinehook/And64InlineHook.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/and64inlinehook/And64InlineHook.hpp
    )
    target_include_directories(and64inlinehook PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}/and64inlinehook
    )
    target_compile_options(and64inlinehook PRIVATE -w)
endif()

if (NOT ZNN_LEGACY)
    add_subdirectory(plti)
    add_subdirectory(urhook)
endif ()
add_subdirectory(frida-gum)

if (ANDROID_ABI STREQUAL "arm64-v8a")
    add_subdirectory(yukizygisk)
endif ()

if (ANDROID_ABI STREQUAL "riscv64")
    add_subdirectory(rv64hook)
        target_compile_options(rv64hook-static PRIVATE -w)
elseif (NOT ZNN_LEGACY)
    option(DOBBY_GENERATE_SHARED "Build Dobby as a shared library" OFF)
    option(BUILD_EXAMPLE "Build Dobby example" OFF)
    option(BUILD_TEST "Build Dobby test" OFF)
    add_subdirectory(dobby)
    target_compile_options(dobby PRIVATE -w)

    add_subdirectory(hookzz)

    add_subdirectory(whale)

    add_subdirectory(yukiplt)

    add_subdirectory(profilo)
endif ()

if (NOT ZNN_LEGACY)
    add_subdirectory(baaadhook)
endif ()

if (NOT ZNN_LEGACY AND (ANDROID_ABI STREQUAL "arm64-v8a" OR ANDROID_ABI STREQUAL "x86_64"))
    add_subdirectory(srx_hook)
    add_subdirectory(srx_inline_hook)
endif ()

add_subdirectory(lzma)
add_subdirectory(xz-embedded)

set(BUILD_SHARED_LIBS OFF)
set(ENABLE_WERROR OFF)
set(ENABLE_TEST OFF)
set(ENABLE_TAR OFF)
set(ENABLE_CPIO OFF)
set(ENABLE_CAT OFF)
set(ENABLE_UNZIP OFF)
set(ENABLE_INSTALL OFF)
set(ENABLE_OPENSSL OFF)
set(ENABLE_NETTLE OFF)
set(ENABLE_LZ4 OFF)
set(ENABLE_LZMA OFF)
set(ENABLE_ZSTD OFF)
set(ENABLE_BZip2 OFF)
set(ENABLE_ZLIB ON)
set(ENABLE_ICONV OFF)
set(ENABLE_ACL OFF)
set(ENABLE_XATTR OFF)

if (ANDROID)
    set(ZNN_LIBRARY_SUFFIXES ${CMAKE_FIND_LIBRARY_SUFFIXES})
    set(CMAKE_FIND_LIBRARY_SUFFIXES .a)
    unset(ZLIB_LIBRARY CACHE)
    unset(ZLIB_LIBRARY_DEBUG CACHE)
    unset(ZLIB_LIBRARY_RELEASE CACHE)
endif ()
add_subdirectory(libarchive)
if (ANDROID)
    set(CMAKE_FIND_LIBRARY_SUFFIXES ${ZNN_LIBRARY_SUFFIXES})
    unset(ZNN_LIBRARY_SUFFIXES)
endif ()

if (TARGET cxx::cxx)
    if (ANDROID_ABI STREQUAL "riscv64")
        target_link_libraries(rv64hook-static cxx::cxx)
    else ()
        target_link_libraries(dobby cxx::cxx)
    endif ()
endif ()
