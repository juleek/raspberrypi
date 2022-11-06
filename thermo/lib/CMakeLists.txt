add_static_library(
   memory.h
   memory.cpp

   TSensorPoller.h
   TSensorPoller.cpp
   TSensorsPoller.h
   TSensorsPoller.cpp

   TPublishItem.h
   TPublishItem.cpp
   ISink.h
   ISink.cpp
   THttpSink.h
   THttpSink.cpp

   TJwt.cpp
   TJwt.h
   TJwt_p.h
   TJwtUpdater.h
   TJwtUpdater.cpp
)

# ============= Qt =============
find_package(Qt6Core CONFIG REQUIRED)
target_link_libraries(${current_target} PUBLIC Qt6::Core)

find_package(Qt6Network CONFIG REQUIRED)
target_link_libraries(${current_target} PRIVATE Qt6::Network)

# ============= OpenSSL =============
find_package(OpenSSL REQUIRED)
target_link_libraries(${current_target} PRIVATE OpenSSL::SSL)
target_link_libraries(${current_target} PRIVATE OpenSSL::Crypto)


add_subdirectory_if_not_added(ut)
