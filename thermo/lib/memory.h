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
         TMakeSmartPtr(Ts &&... Vs) noexcept: Tuple(std::forward<Ts>(Vs)...) {}

         template <class T>
         operator std::unique_ptr<T>() noexcept {
            return std::apply([](auto &&... Vs) { return std::make_unique<T>(std::forward<decltype(Vs)>(Vs)...); },
                              std::move(Tuple));
         }

         template <class T>
         operator std::shared_ptr<T>() noexcept {
            return std::apply([](auto &&... Vs) { return std::make_shared<T>(std::forward<decltype(Vs)>(Vs)...); },
                              std::move(Tuple));
         }

      private:
         std::tuple<Ts...> Tuple;
      };

      template <class... Ts>
      struct TSmartPtr {
         TSmartPtr(Ts &&... Vs) noexcept: Tuple(std::forward<Ts>(Vs)...) {}

         template <class T>
         operator std::unique_ptr<T>() noexcept {
            return std::apply([](auto &&... Vs) { return std::unique_ptr<T>(std::forward<decltype(Vs)>(Vs)...); },
                              std::move(Tuple));
         }

         template <class T>
         operator std::shared_ptr<T>() noexcept {
            return std::apply([](auto &&... Vs) { return std::shared_ptr<T>(std::forward<decltype(Vs)>(Vs)...); },
                              std::move(Tuple));
         }

      private:
         std::tuple<Ts...> Tuple;
      };
   }   // namespace Private


   template <class... Ts>
   Private::TMakeSmartPtr<Ts...> MakeUnique(Ts &&... Vs) noexcept {
      return {std::forward<Ts>(Vs)...};
   }


   template <class... Ts>
   Private::TMakeSmartPtr<Ts...> MakeShared(Ts &&... Vs) noexcept {
      return {std::forward<Ts>(Vs)...};
   }

   // std::unique_ptr<QNetworkReply> NetworkReply = UniquePtr(NetworkAccessManager.get(Request));
   template <class... Ts>
   Private::TSmartPtr<Ts...> UniquePtr(Ts &&... Vs) noexcept {
      return {std::forward<Ts>(Vs)...};
   }

   template <class... Ts>
   Private::TSmartPtr<Ts...> SharedPtr(Ts &&... Vs) noexcept {
      return {std::forward<Ts>(Vs)...};
   }

}   // namespace Util

using Util::MakeShared;
using Util::MakeUnique;

// std::unique_ptr<QNetworkReply> NetworkReply = UniquePtr(NetworkAccessManager.get(Request));
using Util::SharedPtr;
using Util::UniquePtr;
