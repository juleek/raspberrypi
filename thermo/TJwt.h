#pragma once

#include <QString>
#include <memory>

class QIODevice;

class THashData {
public:
   QByteArray Data;

   QString AsHexString() const {
      QString Result = Data.toHex().toLower();
      return Result;
   }
   QByteArray AsBinaryString() const {
      return Data;
   }
   bool IsValid() const {
      return Data.isEmpty() == false;
   }
};
QDebug operator<<(QDebug Out, const THashData &Bits);


enum class TDigestAlgo { SHA256 };

class TDigestSignerPrivate;
class TDigestSigner {
public:
   TDigestSigner(const QByteArray &PrivateKey, const TDigestAlgo Algo);
   ~TDigestSigner();

   // clang-format off
   TDigestSigner &AddData(const char *Data, size_t n) &;
   TDigestSigner &AddData(const QByteArray &Bytes)    &;
   TDigestSigner &AddData(QIODevice &Stream)          &;

   TDigestSigner &&AddData(const char *Data, size_t n) &&;
   TDigestSigner &&AddData(const QByteArray &Bytes)    &&;
   TDigestSigner &&AddData(QIODevice &Stream)          &&;
   // clang-format on

private:
   std::unique_ptr<TDigestSignerPrivate> d;
   friend THashData                      CalculateSignature(TDigestSigner &&Signer);
};


// This operation is NOT reversible, internal state of the hasher will be changed thereafter
THashData CalculateSignature(TDigestSigner &&Signer);
THashData CalculateSignature(const QByteArray &PrivateKey, const QByteArray &String, TDigestAlgo Algo = TDigestAlgo::SHA256);
THashData CalculateSignature(const QByteArray &PrivateKey, QIODevice &Stream, TDigestAlgo Algo = TDigestAlgo::SHA256);



class QDateTime;
class TJwtPrivate;
class TJwt {
public:
   enum TAlgo { // RS256,
      ES256
   };

   ~TJwt();
   TJwt(TJwt &&Another);
   TJwt &operator=(TJwt &&Another);

   TJwt(TAlgo Algo = ES256) noexcept;

   void  SetAlgo(TAlgo Algo);
   TAlgo Algo() const;

   void             SetIssuedAt(const QDateTime &DateTime);
   const QDateTime &IssuedAt() const;

   void             SetExpiration(const QDateTime &DateTime);
   const QDateTime &Expiration() const;

   void           SetAudience(const QString &Audience);
   const QString &Audience();

   QString ComposeToken(QIODevice &Secret) const;

private:
   std::unique_ptr<TJwtPrivate> d;
};
