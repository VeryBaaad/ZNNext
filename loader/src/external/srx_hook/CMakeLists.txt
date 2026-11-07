cmake_minimum_required(VERSION 3.22.1)

if (ANDROID_ABI STREQUAL "arm64-v8a")
    set(SRX_HOOK_FFI_RUST_TARGET "aarch64-linux-android")
elseif (ANDROID_ABI STREQUAL "x86_64")
    set(SRX_HOOK_FFI_RUST_TARGET "x86_64-linux-android")
else ()
    message(FATAL_ERROR "srx_hook does not support ${ANDROID_ABI}")
endif ()

if (NOT EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/srx_hook/Cargo.toml")
    message(FATAL_ERROR
        "srx_hook submodule is missing; run git submodule update --init --recursive")
endif ()

if (NOT CMAKE_ANDROID_API)
    set(CMAKE_ANDROID_API 21)
endif ()

if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(SRX_HOOK_FFI_PROFILE "debug")
    set(SRX_HOOK_FFI_CARGO_FLAGS "")
else ()
    set(SRX_HOOK_FFI_PROFILE "release")
    set(SRX_HOOK_FFI_CARGO_FLAGS "--release")
endif ()

set(SRX_HOOK_FFI_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/cargo")
set(SRX_HOOK_FFI_ARCHIVE
    "${SRX_HOOK_FFI_TARGET_DIR}/${SRX_HOOK_FFI_RUST_TARGET}/${SRX_HOOK_FFI_PROFILE}/libsrx_hook_ffi.a")

file(GLOB_RECURSE SRX_HOOK_FFI_SOURCES CONFIGURE_DEPENDS
    "${CMAKE_CURRENT_SOURCE_DIR}/src/*.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/srx_hook/src/*.rs")
list(APPEND SRX_HOOK_FFI_SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml"
    "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.lock"
    "${CMAKE_CURRENT_SOURCE_DIR}/srx_hook/Cargo.toml")

add_custom_command(
    OUTPUT "${SRX_HOOK_FFI_ARCHIVE}"
    COMMAND "${CMAKE_COMMAND}" -E env
        "ANDROID_NDK_HOME=${ANDROID_NDK}"
        "ANDROID_NDK_ROOT=${ANDROID_NDK}"
        "CARGO_TARGET_DIR=${SRX_HOOK_FFI_TARGET_DIR}"
        cargo ndk -t "${ANDROID_ABI}" -P "${CMAKE_ANDROID_API}" build --locked
            ${SRX_HOOK_FFI_CARGO_FLAGS}
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    DEPENDS ${SRX_HOOK_FFI_SOURCES}
    COMMENT "Building srx_hook FFI for ${ANDROID_ABI}"
    VERBATIM)

add_custom_target(srx_hook_ffi_archive DEPENDS "${SRX_HOOK_FFI_ARCHIVE}")

add_library(srx_hook_ffi STATIC IMPORTED GLOBAL)
set_target_properties(srx_hook_ffi PROPERTIES
    IMPORTED_LOCATION "${SRX_HOOK_FFI_ARCHIVE}"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_SOURCE_DIR}/include")
