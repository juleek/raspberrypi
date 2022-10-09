add_exec(thermo
   main.cpp
)
depends_upon_private(lib)


find_package(Qt6Network CONFIG REQUIRED)
target_link_libraries(${current_target} PRIVATE Qt6::Network)
