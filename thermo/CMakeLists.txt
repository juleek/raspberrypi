cmake_minimum_required(VERSION 3.10 FATAL_ERROR)
get_filename_component(Project ${CMAKE_CURRENT_SOURCE_DIR} NAME)

add_executable(${Project}
   main.cpp
   MakeUnique.h
   TJwt.cpp
   TJwt.h
   TJwt_p.h
   TDriver.cpp
   TDriver.h
   TTempPoller.h
   TTempPoller.cpp
   TGCMqtt.cpp
   TGCMqtt.h   
)

# ============= Qt =============
# Find includes in corresponding build directories
# set(CMAKE_INCLUDE_CURRENT_DIR ON)
# Instruct CMake to run moc automatically when needed
# set(CMAKE_AUTOMOC ON)
# Create code from a list of Qt designer ui files
# set(CMAKE_AUTOUIC ON)
find_package(Qt5Core CONFIG REQUIRED)
target_link_libraries(${Project} PRIVATE Qt5::Core)
find_package(Qt5Network CONFIG REQUIRED)
target_link_libraries(${Project} PRIVATE Qt5::Network)

find_package(FindOpenSSL REQUIRED)
target_link_libraries(${Project} PRIVATE OpenSSL::SSL)
target_link_libraries(${Project} PRIVATE OpenSSL::Crypto)


find_library(QTMQTT_LIB NAMES Qt5Mqtt)
if(QTMQTT_LIB)
   message("Found QTMQTT_LIB: ${QTMQTT_LIB}")
else()
   message(FATAL_ERROR "${QTMQTT_LIB} library not found")
endif()
link_libraries(${TCMALLOC_LIB})


add_link_options(-fuse-ld=lld
)


# target_compile_options
add_compile_options(-Wall
                    -Wextra
                    -pedantic
                    -ftemplate-backtrace-limit=0
)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

