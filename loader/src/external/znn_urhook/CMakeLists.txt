cmake_minimum_required(VERSION 3.22.1)

if (NOT ANDROID_ABI STREQUAL "arm64-v8a")
    return()
endif ()

set(URHOOK_SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../urhook")

add_library(urhook STATIC
    "${CMAKE_CURRENT_SOURCE_DIR}/assembler_wrap.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/disassembler_wrap.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/inline_hook_wrap.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/memory.cpp"
    "${URHOOK_SOURCE_DIR}/src/elf_parser.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/inline_hook_capi.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/plthook_capi.cpp"
)
set_target_properties(urhook PROPERTIES
    CXX_STANDARD 20
    CXX_STANDARD_REQUIRED ON
    POSITION_INDEPENDENT_CODE ON
)

target_include_directories(urhook PUBLIC
    "${URHOOK_SOURCE_DIR}/include"
)
if (TARGET cxx::cxx)
    get_target_property(URHOOK_CXX_INCLUDES cxx::cxx INTERFACE_INCLUDE_DIRECTORIES)
    if (URHOOK_CXX_INCLUDES)
        target_include_directories(urhook SYSTEM PRIVATE ${URHOOK_CXX_INCLUDES})
    endif ()
endif ()

target_compile_options(urhook PRIVATE
    -w
    -fexceptions
    -funwind-tables
)

set_source_files_properties("${CMAKE_CURRENT_SOURCE_DIR}/plthook_capi.cpp"
    PROPERTIES COMPILE_OPTIONS "-fno-exceptions")
