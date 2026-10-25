cmake_minimum_required(VERSION 3.22.1)

if (ANDROID_ABI STREQUAL "riscv64")
    return()
endif ()

set(WHALE_SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../whale/whale")
set(WHALE_SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/znn_whale.cpp"
    "${WHALE_SOURCE_DIR}/src/assembler/assembler.cc"
    "${WHALE_SOURCE_DIR}/src/assembler/memory_region.cc"
    "${WHALE_SOURCE_DIR}/src/platform/memory.cc"
)

if (ANDROID_ABI STREQUAL "arm64-v8a")
    list(APPEND WHALE_SOURCES
        "${WHALE_SOURCE_DIR}/src/dbi/arm64/decoder_arm64.cc"
        "${WHALE_SOURCE_DIR}/src/dbi/arm64/instruction_rewriter_arm64.cc"
        "${WHALE_SOURCE_DIR}/src/dbi/arm64/inline_hook_arm64.cc"
    )
elseif (ANDROID_ABI STREQUAL "armeabi-v7a")
    list(APPEND WHALE_SOURCES
        "${WHALE_SOURCE_DIR}/src/dbi/arm/decoder_arm.cc"
        "${WHALE_SOURCE_DIR}/src/dbi/arm/decoder_thumb.cc"
        "${WHALE_SOURCE_DIR}/src/dbi/arm/instruction_rewriter_arm.cc"
        "${WHALE_SOURCE_DIR}/src/dbi/arm/inline_hook_arm.cc"
    )
elseif (ANDROID_ABI STREQUAL "x86")
    list(APPEND WHALE_SOURCES
        "${WHALE_SOURCE_DIR}/src/assembler/x86/assembler_x86.cc"
        "${WHALE_SOURCE_DIR}/src/assembler/x86/managed_register_x86.cc"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/inline_hook_x86.cc"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/instruction_rewriter_x86.cc"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/decoder.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/distorm.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/instructions.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/insts.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/mnemonics.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/operands.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/prefix.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/textdefs.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/wstring.c"
    )
elseif (ANDROID_ABI STREQUAL "x86_64")
    list(APPEND WHALE_SOURCES
        "${WHALE_SOURCE_DIR}/src/assembler/x86_64/assembler_x86_64.cc"
        "${WHALE_SOURCE_DIR}/src/assembler/x86_64/managed_register_x86_64.cc"
        "${WHALE_SOURCE_DIR}/src/dbi/x86_64/inline_hook_x86_64.cc"
        "${WHALE_SOURCE_DIR}/src/dbi/x86_64/instruction_rewriter_x86_64.cc"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/decoder.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/distorm.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/instructions.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/insts.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/mnemonics.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/operands.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/prefix.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/textdefs.c"
        "${WHALE_SOURCE_DIR}/src/dbi/x86/distorm/wstring.c"
    )
else ()
    return()
endif ()

add_library(whale STATIC ${WHALE_SOURCES})
set_target_properties(whale PROPERTIES
    CXX_STANDARD 14
    CXX_STANDARD_REQUIRED ON
    CXX_EXTENSIONS ON
    POSITION_INDEPENDENT_CODE ON
)
target_include_directories(whale
    PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}"
    PRIVATE
        "${WHALE_SOURCE_DIR}/src"
        "${WHALE_SOURCE_DIR}/src/assembler/vixl"
)
target_include_directories(whale BEFORE PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/compat")
target_compile_options(whale PRIVATE
    -w
    "$<$<COMPILE_LANGUAGE:CXX>:-include>"
    "$<$<COMPILE_LANGUAGE:CXX>:${CMAKE_CURRENT_SOURCE_DIR}/compat/vixl_ostream.h>")

if (TARGET cxx::cxx)
    get_target_property(WHALE_CXX_INCLUDES cxx::cxx INTERFACE_INCLUDE_DIRECTORIES)
    if (WHALE_CXX_INCLUDES)
        target_include_directories(whale SYSTEM PRIVATE ${WHALE_CXX_INCLUDES})
    endif ()
endif ()

if (ANDROID_ABI STREQUAL "arm64-v8a" OR ANDROID_ABI STREQUAL "armeabi-v7a")
    add_subdirectory("${WHALE_SOURCE_DIR}/src/assembler/vixl" vixl)
    target_include_directories(vixl PRIVATE
        "${WHALE_SOURCE_DIR}/src/assembler/vixl")
    if (WHALE_CXX_INCLUDES)
        target_include_directories(vixl SYSTEM PRIVATE ${WHALE_CXX_INCLUDES})
    endif ()
    target_compile_options(vixl PRIVATE
        -w
        "$<$<COMPILE_LANGUAGE:CXX>:-include>"
        "$<$<COMPILE_LANGUAGE:CXX>:${CMAKE_CURRENT_SOURCE_DIR}/compat/vixl_ostream.h>")
    target_link_libraries(whale PRIVATE vixl)
endif ()
