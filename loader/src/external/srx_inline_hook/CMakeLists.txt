cmake_minimum_required(VERSION 3.22.1)

if (ANDROID_ABI STREQUAL "arm64-v8a")
    set(SRX_INLINE_HOOK_FFI_RUST_TARGET "aarch64-linux-android")
elseif (ANDROID_ABI STREQUAL "x86_64")
    set(SRX_INLINE_HOOK_FFI_RUST_TARGET "x86_64-linux-android")
else ()
    message(FATAL_ERROR "srx_inline_hook does not support ${ANDROID_ABI}")
endif ()

if (NOT EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/srx_inline_hook/Cargo.toml")
    message(FATAL_ERROR
        "srx_inline_hook submodule is missing; run git submodule update --init --recursive")
endif ()

set(SRX_INLINE_HOOK_FFI_PLATFORM 31)

set(SRX_INLINE_HOOK_FFI_RUSTFLAGS "-C force-frame-pointers=yes")
if (DEFINED ENV{RUSTFLAGS} AND NOT "$ENV{RUSTFLAGS}" STREQUAL "")
    set(SRX_INLINE_HOOK_FFI_RUSTFLAGS
        "$ENV{RUSTFLAGS} -C force-frame-pointers=yes")
endif ()

if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(SRX_INLINE_HOOK_FFI_PROFILE "debug")
    set(SRX_INLINE_HOOK_FFI_CARGO_FLAGS "")
else ()
    set(SRX_INLINE_HOOK_FFI_PROFILE "release")
    set(SRX_INLINE_HOOK_FFI_CARGO_FLAGS "--release")
endif ()

set(SRX_INLINE_HOOK_FFI_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/cargo")
set(SRX_INLINE_HOOK_FFI_ARCHIVE
    "${SRX_INLINE_HOOK_FFI_TARGET_DIR}/${SRX_INLINE_HOOK_FFI_RUST_TARGET}/${SRX_INLINE_HOOK_FFI_PROFILE}/libsrx_inline_hook_ffi.a")

file(GLOB_RECURSE SRX_INLINE_HOOK_FFI_SOURCES CONFIGURE_DEPENDS
    "${CMAKE_CURRENT_SOURCE_DIR}/src/*.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/srx_inline_hook/src/*.rs")
list(APPEND SRX_INLINE_HOOK_FFI_SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/CMakeLists.txt"
    "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml"
    "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.lock"
    "${CMAKE_CURRENT_SOURCE_DIR}/srx_inline_hook/Cargo.toml"
    "${CMAKE_CURRENT_SOURCE_DIR}/srx_inline_hook/build.rs")

add_custom_command(
    OUTPUT "${SRX_INLINE_HOOK_FFI_ARCHIVE}"
    COMMAND "${CMAKE_COMMAND}" -E env
        "ANDROID_NDK_HOME=${ANDROID_NDK}"
        "ANDROID_NDK_ROOT=${ANDROID_NDK}"
        "CARGO_TARGET_DIR=${SRX_INLINE_HOOK_FFI_TARGET_DIR}"
        "RUSTFLAGS=${SRX_INLINE_HOOK_FFI_RUSTFLAGS}"
        cargo ndk -t "${ANDROID_ABI}" -P "${SRX_INLINE_HOOK_FFI_PLATFORM}" build
            ${SRX_INLINE_HOOK_FFI_CARGO_FLAGS}
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    DEPENDS ${SRX_INLINE_HOOK_FFI_SOURCES}
    COMMENT "Building srx_inline_hook FFI for ${ANDROID_ABI}"
    VERBATIM)

add_custom_target(srx_inline_hook_ffi_archive DEPENDS "${SRX_INLINE_HOOK_FFI_ARCHIVE}")

add_library(srx_inline_hook_ffi STATIC IMPORTED GLOBAL)
set_target_properties(srx_inline_hook_ffi PROPERTIES
    IMPORTED_LOCATION "${SRX_INLINE_HOOK_FFI_ARCHIVE}"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_SOURCE_DIR}/include")
