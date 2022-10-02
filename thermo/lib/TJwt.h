#pragma once

#include <QDateTime>
#include <QString>
#include <memory>

class QIODevice;

class THashData {
public:
   QByteArray Data;

   QString ToHexString() const {
      QString Result = Data.toHex().toLower();
      return Result;
   }
   QByteArray ToBinaryString() const {
      return Data;
   }
   QByteArray ToBase64Url() const {
      return Data.toBase64(QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals);
   }
   bool IsValid() const {
      return Data.isEmpty() == false;
   }
};
QDebug operator<<(QDebug Out, const THashData &Bits);


enum class TDigestAlgo { SHA256 };
enum class TKeyType { EC, RSA, Any };

class TDigestCalculatorPrivate;
class TDigestCalculator {
public:
   TDigestCalculator(const TDigestAlgo Algo);
   ~TDigestCalculator();

   // clang-format off
   TDigestCalculator &AddData(const char *Data, size_t n) &;
   TDigestCalculator &AddData(const QByteArray &Bytes)    &;
   TDigestCalculator &AddData(QIODevice &Stream)          &;

   TDigestCalculator &&AddData(const char *Data, size_t n) &&;
   TDigestCalculator &&AddData(const QByteArray &Bytes)    &&;
   TDigestCalculator &&AddData(QIODevice &Stream)          &&;
   // clang-format on

private:
   std::unique_ptr<TDigestCalculatorPrivate> d;
   friend THashData CalculateSignature(TDigestCalculator &&Signer, const QByteArray &PrivateKey, const TKeyType KeyType);
};

// This operation is NOT reversible, internal state of the hasher will be changed thereafter
THashData CalculateSignature(TDigestCalculator &&Signer, const QByteArray &PrivateKey, const TKeyType KeyType);
THashData CalculateSignature(const QByteArray &PrivateKey,
                             const QByteArray &Data,
                             const TKeyType    KeyType,
                             const TDigestAlgo DigestAlgo);
THashData CalculateSignature(const QByteArray &PrivateKey, QIODevice &Data, const TKeyType KeyType, const TDigestAlgo DigestAlgo);



class QDateTime;
class TJwtPrivate;
class TJwt {
public:
   enum TAlgo { ES256, RS256 };

   ~TJwt();
   TJwt(TJwt &&Another);
   TJwt &operator=(TJwt &&Another);

   TJwt(TAlgo Algo = ES256) noexcept;

   void  SetAlgo(TAlgo Algo);
   TAlgo Algo() const;

   void             SetIssuedAt(const QDateTime &DateTime = QDateTime::currentDateTime());
   const QDateTime &IssuedAt() const;

   void             SetExpiration(const QDateTime &DateTime);
   const QDateTime &Expiration() const;

   void           SetAudience(const QString &Audience);
   const QString &Audience();

   void          SetTargetAudience(const QString &TargetAudience);
   const QString TargetAudience();

   void          SetIss(const QString &TargetAudience);
   const QString Iss();

   void          SetSub(const QString &TargetAudience);
   const QString Sub();

   QString ComposeToken(QIODevice &Secret) const;

private:
   std::unique_ptr<TJwtPrivate> d;
};
