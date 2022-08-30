# ============================================================================================================
# ============================================================================================================
# Implementation part. See public API below

set(processed_paths "" CACHE INTERNAL "")

macro(set_if_not_set variable)
   if(NOT DEFINED ${variable})
      set(${variable} ${ARGN})
   endif()
endmacro()


function(get_target_name_for directory ret)
   file(RELATIVE_PATH relative_path ${PROJECT_SOURCE_DIR} ${directory})
   string(REPLACE "/" "-" dashed_name ${relative_path})
   set(${ret} "${dashed_name}" PARENT_SCOPE)
endfunction()


# ============================================================================================================
# ============================================================================================================
# Public API


macro(add_static_library)
   get_target_name_for(${CMAKE_CURRENT_SOURCE_DIR} default_target_name)
   set_if_not_set(current_target ${default_target_name})
   message("SubProjectBuilder: add_static_library: Adding STATIC library ${current_target}, with files: ${ARGV}")
   add_library(${current_target} STATIC ${ARGV})
   set_property(TARGET ${current_target} PROPERTY POSITION_INDEPENDENT_CODE ON)
endmacro()



# Add an cmake's interface library (a header-only library)
macro(add_interface_library)
   get_target_name_for(${CMAKE_CURRENT_SOURCE_DIR} default_target_name)
   set_if_not_set(current_target ${default_target_name})
   message("SubProjectBuilder: add_static_library: Adding INTERFACE library ${current_target}, with files: ${ARGV}")
   add_library(${current_target} INTERFACE 
      # ${ARGV}
   )
   # set_property(TARGET ${current_target} PROPERTY POSITION_INDEPENDENT_CODE ON)
endmacro()



macro(add_exec name)
   set(current_target ${name})
   message("SubProjectBuilder: add_exec: Adding executable ${ARGV}")
   add_executable(${ARGV})
endmacro()



# Same as executable, but also linked against google-test libs
macro(add_unit_test)
   get_target_name_for(${CMAKE_CURRENT_SOURCE_DIR} default_target_name)
   set_if_not_set(current_target ${default_target_name})
   message("SubProjectBuilder: add_unit_test: Adding unit-test ${ARGV}")
   add_executable(${current_target} ${ARGV})
   target_link_libraries(${current_target} PUBLIC gtest gmock gtest_main)
endmacro()



# This is used only if you want to have an alias (another name) for already existing target/library.
macro(add_alias_library target_name)
   get_target_name_for(${CMAKE_CURRENT_SOURCE_DIR} default_target_name)
   set_if_not_set(current_target ${default_target_name})
   
   get_target_property(destination_target ${target_name} ALIASED_TARGET)
   if(destination_target)
      message("SubProjectBuilder: add_alias_library: Adding ALIAS library ${current_target} for library ${target_name}, which is itself an alias for ${destination_target}")
   else()
      set(destination_target ${target_name})
      message("SubProjectBuilder: add_alias_library: Adding ALIAS library ${current_target} for library ${destination_target}")
   endif()

   add_library(${current_target} ALIAS ${destination_target})
endmacro()



# Adds directory. Directory path starts from the root/project CMakeLists.txt
function(add_directory_if_not_added directory)
   # message("SubProjectBuilder: add_directory_if_not_added: ARGV: ${ARGV}, processed_paths: ${processed_paths}")
   get_target_name_for(${PROJECT_SOURCE_DIR}/${directory} dashed_name)
   if("${processed_paths}" MATCHES "X${dashed_name}X")
      # message("SubProjectBuilder: add_directory_if_not_added: dashed_name: ${dashed_name} found in the list of processed paths, ignoring it...")
      return()
   endif()

   # message("SubProjectBuilder: add_directory_if_not_added: dashed_name: ${dashed_name} was not found in the list of processed paths, adding it...")
   set(local_processed_paths ${processed_paths})
   list(APPEND local_processed_paths "X${dashed_name}X")
   # message("SubProjectBuilder: add_directory_if_not_added: local_processed_paths: ${local_processed_paths}")
   set(processed_paths "${local_processed_paths}" CACHE INTERNAL "")

   set(backup_current_target ${current_target})
   unset(current_target)
   message("SubProjectBuilder: add_directory_if_not_added: recursing to: ${directory}")
   add_subdirectory(${PROJECT_SOURCE_DIR}/${directory} ${PROJECT_BINARY_DIR}/${directory})
   set(current_target ${backup_current_target})
endfunction()



# Adds *sub*directory. Directory path starts from the **current** CMakeLists.txt
function(add_subdirectory_if_not_added subdir)
   file(RELATIVE_PATH relative_to_root ${PROJECT_SOURCE_DIR} ${CMAKE_CURRENT_SOURCE_DIR}/${subdir})
   message("SubProjectBuilder: add_subdirectory_if_not_added: relative_to_root: ${relative_to_root}")
   add_directory_if_not_added(${relative_to_root})
endfunction()



# Use this to specify that current library/executable depends upon a library in a given directory.
# Directory path starts from the root CMakeLists.txt
function(depends_upon paths)
   # message("SubProjectBuilder: depends_upon: name: ${current_target}, path: ${path}")
   foreach(path IN LISTS ARGV)
      add_directory_if_not_added(${path})
      get_target_name_for(${PROJECT_SOURCE_DIR}/${path} dashed_name)
      target_link_libraries(${current_target} ${dashed_name})
   endforeach()
endfunction()



# Use this to specify that current library/executable depends upon a library in a given directory.
# Directory path starts from the root CMakeLists.txt
# cmake's PUBLIC modifiers is used.
function(depends_upon_public paths)
   # message("SubProjectBuilder: depends_upon: name: ${current_target}, path: ${path}")
   foreach(path IN LISTS ARGV)
      add_directory_if_not_added(${path})
      get_target_name_for(${PROJECT_SOURCE_DIR}/${path} dashed_name)
      target_link_libraries(${current_target} PUBLIC ${dashed_name})
   endforeach()
endfunction()



# Use this to specify that current library/executable depends upon a library in a given directory.
# Directory path starts from the root CMakeLists.txt
# cmake's PRIVATE modifiers is used.
function(depends_upon_private paths)
   # message("SubProjectBuilder: depends_upon: name: ${current_target}, path: ${path}")
   foreach(path IN LISTS ARGV)
      add_directory_if_not_added(${path})
      get_target_name_for(${PROJECT_SOURCE_DIR}/${path} dashed_name)
      target_link_libraries(${current_target} PRIVATE ${dashed_name})
   endforeach()
endfunction()



# Use this if you want to link old-style cmake target against new-style cmake library (in a given directory)
function(target_link_against_dir target directory)
   add_directory_if_not_added(${directory})
   get_target_name_for(${PROJECT_SOURCE_DIR}/${directory} target_name)
   list(LENGTH ARGN num_of_optional_args)
   if("${num_of_optional_args}" EQUAL "0")
      message("SubProjectBuilder: linking ${target} against ${target_name}")
      target_link_libraries(${target} ${target_name})
   else()
      list(GET ARGN 0 modifier)
      message("SubProjectBuilder: linking ${target} against ${target_name} with ${modifier} modifier")
      target_link_libraries(${target} ${modifier} ${target_name})
   endif()
endfunction()
