# https://stackoverflow.com/questions/3766740/overriding-a-default-option-value-in-cmake-from-a-parent-cmakelists-txt
SET(INSTALL_GTEST OFF CACHE BOOL "Build libfoo shared library") # Otherwise CMake Error: install(EXPORT "GTestTargets" ...) includes target "gtest" which requires target "cxx_static" that is not in the export set
add_subdirectory_if_not_added(googletest)
