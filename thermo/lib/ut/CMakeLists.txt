add_unit_test(
   THttpSink.t.cpp
   TJwt.t.cpp
)
depends_upon_public(
   lib
)
target_compile_options(${current_target} PRIVATE -Werror)
