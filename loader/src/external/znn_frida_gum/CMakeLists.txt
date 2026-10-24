cmake_minimum_required(VERSION 3.22.1)

if (ANDROID_ABI STREQUAL "riscv64")
    return()
endif ()

set(FRIDA_GUM_SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../frida-gum")
set(FRIDA_GUM_BUILD_DIR "${CMAKE_CURRENT_BINARY_DIR}/frida-gum-${ANDROID_ABI}")
set(FRIDA_GUM_DEPS_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../../../.build/frida-gum-deps"
    CACHE PATH "Shared Frida dependency cache")

if (ANDROID_ABI STREQUAL "arm64-v8a")
    set(FRIDA_GUM_HOST "android-arm64")
elseif (ANDROID_ABI STREQUAL "armeabi-v7a")
    set(FRIDA_GUM_HOST "android-arm")
elseif (ANDROID_ABI STREQUAL "x86")
    set(FRIDA_GUM_HOST "android-x86")
elseif (ANDROID_ABI STREQUAL "x86_64")
    set(FRIDA_GUM_HOST "android-x86_64")
else ()
    message(FATAL_ERROR "frida-gum does not support Android ABI ${ANDROID_ABI}")
endif ()

if (CMAKE_HOST_WIN32)
    set(FRIDA_GUM_BUILD_OS "windows")
elseif (APPLE)
    set(FRIDA_GUM_BUILD_OS "macos")
else ()
    set(FRIDA_GUM_BUILD_OS "linux")
endif ()
if (CMAKE_HOST_SYSTEM_PROCESSOR MATCHES "^(x86_64|AMD64|amd64)$")
    set(FRIDA_GUM_BUILD_ARCH "x86_64")
elseif (CMAKE_HOST_SYSTEM_PROCESSOR MATCHES "^(aarch64|arm64|ARM64)$")
    set(FRIDA_GUM_BUILD_ARCH "arm64")
elseif (CMAKE_HOST_SYSTEM_PROCESSOR MATCHES "^(i[3-6]86|x86)$")
    set(FRIDA_GUM_BUILD_ARCH "x86")
else ()
    message(FATAL_ERROR "frida-gum: unsupported build host ${CMAKE_HOST_SYSTEM_PROCESSOR}")
endif ()
set(FRIDA_GUM_BUILD_MACHINE "${FRIDA_GUM_BUILD_OS}-${FRIDA_GUM_BUILD_ARCH}")

if (NOT CMAKE_ANDROID_NDK)
    set(CMAKE_ANDROID_NDK "$ENV{ANDROID_NDK_ROOT}")
endif ()
if (NOT CMAKE_ANDROID_NDK)
    set(CMAKE_ANDROID_NDK "$ENV{ANDROID_NDK_HOME}")
endif ()

find_package(Python3 COMPONENTS Interpreter REQUIRED)

if (CMAKE_MAKE_PROGRAM MATCHES "ninja")
    set(FRIDA_GUM_NINJA "${CMAKE_MAKE_PROGRAM}")
else ()
    find_program(FRIDA_GUM_NINJA NAMES ninja REQUIRED)
endif ()

if (NOT CMAKE_STRIP)
    find_program(FRIDA_GUM_STRIP NAMES llvm-strip strip REQUIRED)
else ()
    set(FRIDA_GUM_STRIP "${CMAKE_STRIP}")
endif ()

set(FRIDA_GUM_LIBRARY "${FRIDA_GUM_BUILD_DIR}/gum/devkit/libfrida-gum.a")
set(FRIDA_GUM_ENUMS "${FRIDA_GUM_BUILD_DIR}/gum/gumenumtypes.h")
set(FRIDA_GUM_SDK_DIR "${FRIDA_GUM_DEPS_DIR}/sdk-${FRIDA_GUM_HOST}")
set(FRIDA_GUM_INCLUDE_DIRS
    "${FRIDA_GUM_SOURCE_DIR}"
    "${FRIDA_GUM_BUILD_DIR}"
    "${FRIDA_GUM_SDK_DIR}/include/capstone"
    "${FRIDA_GUM_SDK_DIR}/include/glib-2.0"
    "${FRIDA_GUM_SDK_DIR}/lib/glib-2.0/include"
)
file(MAKE_DIRECTORY ${FRIDA_GUM_INCLUDE_DIRS})

add_custom_command(
    OUTPUT "${FRIDA_GUM_LIBRARY}" "${FRIDA_GUM_ENUMS}"
    COMMAND
        "${Python3_EXECUTABLE}" "${CMAKE_CURRENT_SOURCE_DIR}/build.py"
        --source "${FRIDA_GUM_SOURCE_DIR}"
        --build "${FRIDA_GUM_BUILD_DIR}"
        --deps "${FRIDA_GUM_DEPS_DIR}"
        --host "${FRIDA_GUM_HOST}"
        --build-machine "${FRIDA_GUM_BUILD_MACHINE}"
        --ndk "${CMAKE_ANDROID_NDK}"
        --ninja "${FRIDA_GUM_NINJA}"
        --strip "${FRIDA_GUM_STRIP}"
    DEPENDS
        "${CMAKE_CURRENT_SOURCE_DIR}/build.py"
        "${FRIDA_GUM_SOURCE_DIR}/meson.build"
        "${FRIDA_GUM_SOURCE_DIR}/meson.options"
        "${FRIDA_GUM_SOURCE_DIR}/releng/deps.toml"
    COMMENT "Building frida-gum for ${ANDROID_ABI}"
    VERBATIM
)

add_custom_target(frida_gum_build DEPENDS "${FRIDA_GUM_LIBRARY}" "${FRIDA_GUM_ENUMS}")

add_library(frida-gum STATIC IMPORTED GLOBAL)
set_target_properties(frida-gum PROPERTIES
    IMPORTED_LOCATION "${FRIDA_GUM_LIBRARY}"
    INTERFACE_INCLUDE_DIRECTORIES "${FRIDA_GUM_INCLUDE_DIRS}"
    INTERFACE_SYSTEM_INCLUDE_DIRECTORIES "${FRIDA_GUM_INCLUDE_DIRS}"
    INTERFACE_LINK_LIBRARIES "log;m;dl"
)
