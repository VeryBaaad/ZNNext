cmake_minimum_required(VERSION 3.22.1)

set(YUKIZYGISK_IHOOK_SRC
    ${CMAKE_CURRENT_SOURCE_DIR}/src/userspace/zygisk/core/src)

enable_language(ASM)

add_library(yukiihook STATIC
    ${YUKIZYGISK_IHOOK_SRC}/self_unmap.S
)

set_target_properties(yukiihook PROPERTIES
    POSITION_INDEPENDENT_CODE ON
    C_VISIBILITY_PRESET hidden
    VISIBILITY_INLINES_HIDDEN ON)
target_include_directories(yukiihook PUBLIC ${YUKIZYGISK_IHOOK_SRC})
