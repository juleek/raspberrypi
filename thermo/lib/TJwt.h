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



// https://developers.google.com/identity/protocols/oauth2/service-account#authorizingrequests
class TJwt {
public:
   enum TAlgo { ES256, RS256 };

   TJwt::TAlgo Algo;
   QString     Audience;
   QString     TargetAudience;
   QString     Sub;
   QString     Iss;
   QString     Scopes;

   QDateTime IssuedAt   = QDateTime::currentDateTime();
   QDateTime Expiration = QDateTime::currentDateTime().addSecs(60 * 60);

   QString ComposeSignedToken(QIODevice &Secret) const;
};
