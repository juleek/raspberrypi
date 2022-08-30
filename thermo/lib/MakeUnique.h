#pragma once

#include <memory>

namespace NPrivate {
   template <class X>
   struct TUniquePtrInitHelper {
      TUniquePtrInitHelper(X *Raw) noexcept {
         m_Raw = Raw;
      }
      template <class T, class D>
      operator std::unique_ptr<T, D>() const noexcept {
         return std::unique_ptr<T, D>(m_Raw);
      }

   private:
      X *m_Raw;
   };
} // namespace NPrivate

template <class X>
NPrivate::TUniquePtrInitHelper<X> MakeUnique(X *Raw) noexcept {
   return {Raw};
}
