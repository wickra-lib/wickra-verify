# Build and run the C and C++ examples against the wickra-verify C ABI.
#
#   cargo build --release -p wickra-verify-c
#   cmake -S examples/c -B examples/c/build
#   cmake --build examples/c/build --config Release
#   ctest --test-dir examples/c/build -C Release --output-on-failure
#
# WICKRA_VERIFY_LIB_DIR defaults to the workspace target/release directory and
# can be overridden for out-of-tree builds.

cmake_minimum_required(VERSION 3.15)
project(wickra_verify_examples C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(WICKRA_VERIFY_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release"
    CACHE PATH "Directory containing the built wickra-verify C ABI library")

set(WICKRA_VERIFY_INCLUDE "${CMAKE_CURRENT_SOURCE_DIR}/../../bindings/c/include")

find_library(WICKRA_VERIFY_LIB
    NAMES wickra_verify libwickra_verify
    HINTS "${WICKRA_VERIFY_LIB_DIR}"
    REQUIRED)

add_executable(verify verify.c)
target_include_directories(verify PRIVATE "${WICKRA_VERIFY_INCLUDE}")
target_link_libraries(verify PRIVATE "${WICKRA_VERIFY_LIB}")

add_executable(verifypp verify.cpp)
target_include_directories(verifypp PRIVATE "${WICKRA_VERIFY_INCLUDE}")
target_link_libraries(verifypp PRIVATE "${WICKRA_VERIFY_LIB}")

# On Windows the runtime DLL must sit next to the executable (there is no rpath);
# without it the loader pops a modal "DLL not found" dialog that hangs ctest. Copy
# it next to each example after build. (Unix resolves the .so/.dylib via the
# linker's embedded path, so this is Windows-only.)
if(WIN32)
    foreach(example verify verifypp)
        add_custom_command(TARGET ${example} POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E copy_if_different
                "${WICKRA_VERIFY_LIB_DIR}/wickra_verify.dll"
                "$<TARGET_FILE_DIR:${example}>"
            VERBATIM)
    endforeach()
endif()

enable_testing()
add_test(NAME verify COMMAND verify)
add_test(NAME verifypp COMMAND verifypp)
# A missing runtime dependency must fail fast, never block the runner for hours.
set_tests_properties(verify verifypp PROPERTIES TIMEOUT 60)
