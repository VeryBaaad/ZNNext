cmake_minimum_required(VERSION 3.22.1)

include(${CMAKE_CURRENT_SOURCE_DIR}/../yukiplt/cmake/YukiPLTSources.cmake)

add_library(yukiplt STATIC ${YUKIPLT_SOURCES})
target_compile_features(yukiplt PUBLIC cxx_std_17)
set_target_properties(yukiplt PROPERTIES
    CXX_STANDARD 17 CXX_STANDARD_REQUIRED ON CXX_EXTENSIONS OFF
    POSITION_INDEPENDENT_CODE ON
    CXX_VISIBILITY_PRESET hidden C_VISIBILITY_PRESET hidden VISIBILITY_INLINES_HIDDEN ON)
target_include_directories(yukiplt PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/../yukiplt/include)
target_compile_definitions(yukiplt PUBLIC YUKIPLT_STATIC=1)

target_compile_options(yukiplt PRIVATE -fexceptions -funwind-tables)

if (TARGET cxx::cxx)
    get_target_property(YUKIPLT_CXX_INCLUDES cxx::cxx INTERFACE_INCLUDE_DIRECTORIES)
    if (YUKIPLT_CXX_INCLUDES)
        target_include_directories(yukiplt SYSTEM PRIVATE ${YUKIPLT_CXX_INCLUDES})
    endif ()
endif ()

find_package(Threads REQUIRED)
target_link_libraries(yukiplt PRIVATE Threads::Threads ${CMAKE_DL_LIBS})
