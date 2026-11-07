cmake_minimum_required(VERSION 3.22.1)

set(PROFILO_SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/src)
set(PROFILO_DEPS_DIR ${PROFILO_SOURCE_DIR}/deps)

if (ANDROID_ABI STREQUAL "arm64-v8a")
    set(PROFILO_TRAMPOLINE ${PROFILO_DEPS_DIR}/plthooks/trampoline_arm64.c)
elseif (ANDROID_ABI STREQUAL "armeabi-v7a")
    set(PROFILO_TRAMPOLINE ${PROFILO_DEPS_DIR}/plthooks/trampoline_arm.c)
elseif (ANDROID_ABI STREQUAL "x86_64")
    set(PROFILO_TRAMPOLINE ${PROFILO_DEPS_DIR}/plthooks/trampoline_x86_64.c)
elseif (ANDROID_ABI STREQUAL "x86")
    set(PROFILO_TRAMPOLINE ${PROFILO_DEPS_DIR}/plthooks/trampoline_x86.c)
else ()
    message(FATAL_ERROR "profilo does not support ${ANDROID_ABI}")
endif ()

add_library(profilo STATIC
    ${PROFILO_DEPS_DIR}/plthooks/hooks.cpp
    ${PROFILO_DEPS_DIR}/plthooks/plthooks.cpp
    ${PROFILO_DEPS_DIR}/plthooks/trampoline.cpp
    ${PROFILO_TRAMPOLINE}
    ${PROFILO_DEPS_DIR}/linker/elfSharedLibData.cpp
    ${PROFILO_DEPS_DIR}/linker/linker.cpp
    ${PROFILO_DEPS_DIR}/linker/sharedlibs.cpp
    ${PROFILO_DEPS_DIR}/sig_safe_write/sig_safe_write.c
    ${PROFILO_DEPS_DIR}/sigmux/phaser.c
    ${PROFILO_DEPS_DIR}/sigmux/sigmux.c
    ${PROFILO_DEPS_DIR}/build/build.cpp
    ${PROFILO_DEPS_DIR}/breakpad/abort_with_reason.c
    ${PROFILO_DEPS_DIR}/fb/log.cpp
)
set_target_properties(profilo PROPERTIES
    POSITION_INDEPENDENT_CODE ON
)
target_include_directories(profilo PUBLIC
    ${PROFILO_DEPS_DIR}
    ${PROFILO_DEPS_DIR}/breakpad
    ${PROFILO_DEPS_DIR}/lss
    ${PROFILO_DEPS_DIR}/sig_safe_write
    ${PROFILO_DEPS_DIR}/sigmux
)
target_compile_definitions(profilo PRIVATE LOG_TAG="ZNN/profilo")
target_compile_options(profilo PRIVATE
    -w
    -fexceptions
    -funwind-tables
    $<$<COMPILE_LANGUAGE:C>:-std=gnu18>)

if (TARGET cxx::cxx)
    get_target_property(PROFILO_CXX_INCLUDES cxx::cxx INTERFACE_INCLUDE_DIRECTORIES)
    if (PROFILO_CXX_INCLUDES)
        target_include_directories(profilo SYSTEM PRIVATE ${PROFILO_CXX_INCLUDES})
    endif ()
endif ()

find_package(Threads REQUIRED)
target_link_libraries(profilo PRIVATE Threads::Threads ${CMAKE_DL_LIBS})
