#pragma once

#include <memory>
#include <stddef.h>
#include <tuple>
#include <utility>

namespace Util {

   /// ------------------------- MakeUnique -------------------------

   namespace Private {
      template <class... Ts>
      struct TMakeSmartPtr {
         TMakeSmartPtr(Ts &&...Vs) noexcept: Tuple(std::forward<Ts>(Vs)...) {}

         template <class T, class Deleter>
         operator std::unique_ptr<T, Deleter>() noexcept {
            return std::apply([](auto &&...Vs) { return std::unique_ptr<T, Deleter>(new T(std::forward<decltype(Vs)>(Vs)...)); },
                              std::move(Tuple));
         }

      private:
         std::tuple<Ts...> Tuple;
      };

      template <class... Ts>
      struct TSmartPtr {
         TSmartPtr(Ts &&...Vs) noexcept: Tuple(std::forward<Ts>(Vs)...) {}

         template <class T, class Deleter>
         operator std::unique_ptr<T, Deleter>() noexcept {
            return std::apply([](auto &&...Vs) { return std::unique_ptr<T, Deleter>(std::forward<decltype(Vs)>(Vs)...); },
                              std::move(Tuple));
         }

      private:
         std::tuple<Ts...> Tuple;
      };
   }   // namespace Private


   template <class... Ts>
   Private::TMakeSmartPtr<Ts...> MakeUnique(Ts &&...Vs) noexcept {
      return {std::forward<Ts>(Vs)...};
   }

   // std::unique_ptr<QNetworkReply> NetworkReply = UniquePtr(NetworkAccessManager.get(Request));
   template <class... Ts>
   Private::TSmartPtr<Ts...> UniquePtr(Ts &&...Vs) noexcept {
      return {std::forward<Ts>(Vs)...};
   }

}   // namespace Util

using Util::MakeUnique;

// std::unique_ptr<QNetworkReply> NetworkReply = UniquePtr(NetworkAccessManager.get(Request));
using Util::UniquePtr;
