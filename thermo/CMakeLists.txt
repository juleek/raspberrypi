project(thermo CXX C)
cmake_minimum_required(VERSION 3.10 FATAL_ERROR)
set(CMAKE_MODULE_PATH ${CMAKE_SOURCE_DIR}/CMake)
include(subproject_builder)


# =================================================================
# C++ setup

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# add_link_options(-fuse-ld=lld)
add_compile_options(
   -Wall
   -Wextra
   -pedantic
   -ftemplate-backtrace-limit=0
)


# =================================================================
# Qt setup

# Find includes in corresponding build directories
set(CMAKE_INCLUDE_CURRENT_DIR ON)
# Instruct CMake to run moc automatically when needed
set(CMAKE_AUTOMOC ON)
# Create code from a list of Qt designer ui files
# set(CMAKE_AUTOUIC ON)


# =================================================================
# The app itself:

add_directory_if_not_added(contrib/googletest)
add_subdirectory_if_not_added(app)
