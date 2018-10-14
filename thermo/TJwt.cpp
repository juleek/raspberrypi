#include "TJwt.h"
#include "TJwt_p.h"

#include <QIODevice>
#include <QJsonDocument>
#include <QJsonObject>
#include <QtDebug>
#include <cstring>

#include <openssl/bio.h>
#include <openssl/crypto.h>
#include <openssl/hmac.h>
#include <openssl/pem.h>
#include <openssl/sha.h>

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
template <class X>
TUniquePtrInitHelper<X> MakeUnique(X *Raw) noexcept {
   return {Raw};
}



QDebug operator<<(QDebug Out, const THashData &Bits) {
   Out << Bits.AsHexString();
   return Out;
}

struct TOpenSSLDeleter {
   void operator()(EVP_MD_CTX *Ctx) {
      EVP_MD_CTX_destroy(Ctx);
   }
   void operator()(EVP_PKEY *Key) {
      EVP_PKEY_free(Key);
   }
   void operator()(EC_KEY *Key) {
      EC_KEY_free(Key);
   }
};
class TDigestSignerPrivate {
public:
   TDigestSignerPrivate() {
      Ctx = MakeUnique(EVP_MD_CTX_create());
      if (Ctx == nullptr)
         Ok = false;
   }
   std::unique_ptr<EVP_MD_CTX, TOpenSSLDeleter> Ctx;
   bool                                         Ok = true;
};

namespace {
   const EVP_MD *AlgoToOpenSSL(const TDigestAlgo Algo) {
      switch (Algo) {
         case TDigestAlgo::SHA256: return EVP_sha256();
      }
   }
} // namespace



TDigestSigner::TDigestSigner(const QByteArray &PrivateKey, const TDigestAlgo Algo)
    : d(new TDigestSignerPrivate) {
   if (!d->Ok)
      return;

   BIO *Bio = BIO_new_mem_buf(PrivateKey.data(), PrivateKey.size());
   if (Bio == nullptr) {
      d->Ok = false;
      return;
   }

   std::unique_ptr<EC_KEY, TOpenSSLDeleter> ECKey = MakeUnique(PEM_read_bio_ECPrivateKey(Bio, nullptr, nullptr, nullptr));
   if (ECKey == nullptr) {
      d->Ok = false;
      return;
   }

   std::unique_ptr<EVP_PKEY, TOpenSSLDeleter> EVPKey = MakeUnique(EVP_PKEY_new());
   if (EVPKey == nullptr) {
      d->Ok = false;
      return;
   }

   d->Ok = EVP_PKEY_set1_EC_KEY(EVPKey.get(), ECKey.get());
   if (!d->Ok)
      return;

   d->Ok = EVP_DigestSignInit(d->Ctx.get(), nullptr, AlgoToOpenSSL(Algo), nullptr, EVPKey.get());
   if (!d->Ok)
      return;
}
TDigestSigner::~TDigestSigner() = default;

TDigestSigner &TDigestSigner::AddData(const char *Data, size_t n) & {
   if (d->Ok == false)
      return *this;

   d->Ok = EVP_DigestSignUpdate(d->Ctx.get(), Data, n);

   return *this;
}
TDigestSigner &TDigestSigner::AddData(const QByteArray &Bytes) & {
   return AddData(Bytes.data(), Bytes.size());
}
TDigestSigner &TDigestSigner::AddData(QIODevice &Stream) & {
   static const constexpr size_t BUFFER_SIZE = 1 * 1024 * 1024;
   QByteArray                    buffer;
   buffer.resize(BUFFER_SIZE);

   for (; d->Ok;) {
      const size_t Actual = Stream.read(const_cast<char *>(buffer.data()), buffer.size());
      if (Actual == 0)
         break;
      AddData(buffer.data(), Actual);
   }
   return *this;
}

TDigestSigner &&TDigestSigner::AddData(const char *Data, size_t n) && {
   return std::move(AddData(Data, n));
}
TDigestSigner &&TDigestSigner::AddData(const QByteArray &String) && {
   return std::move(AddData(String));
}
TDigestSigner &&TDigestSigner::AddData(QIODevice &Stream) && {
   return std::move(AddData(Stream));
}

THashData CalculateSignature(TDigestSigner &&Signer) {
   if (Signer.d->Ok == false)
      return {};

   bool   Ok;
   size_t SignatureLength;
   Ok = EVP_DigestSignFinal(Signer.d->Ctx.get(), nullptr, &SignatureLength);
   if (!Ok)
      return {};
   THashData Result;
   Result.Data.resize(SignatureLength);

   Ok = EVP_DigestSignFinal(Signer.d->Ctx.get(), reinterpret_cast<unsigned char *>(Result.Data.data()), &SignatureLength);
   if (!Ok)
      return {};

   return Result;
}

THashData CalculateSignature(const QByteArray &PrivateKey, const QByteArray &String, TDigestAlgo Algo) {
   return CalculateSignature(TDigestSigner(PrivateKey, Algo).AddData(String));
}
THashData CalculateSignature(const QByteArray &PrivateKey, QIODevice &Stream, TDigestAlgo Algo) {
   return CalculateSignature(TDigestSigner(PrivateKey, Algo).AddData(Stream));
}



// ========================================================================================================================
// ========================================================================================================================
// ========================================================================================================================

TJwt::~TJwt() = default;

TJwt::TJwt(TJwt &&Another) {
   *this = std::move(Another);
}
TJwt &TJwt::operator=(TJwt &&Another) {
   if (this == &Another)
      return *this;
   d = std::move(Another.d);
   return *this;
}
TJwt::TJwt(TAlgo Algo) noexcept {
   d = std::make_unique<TJwtPrivate>();
   SetAlgo(Algo);
}
void TJwt::SetAlgo(TAlgo Algo) {
   d->Algo = Algo;
}
TJwt::TAlgo TJwt::Algo() const {
   return d->Algo;
}
void TJwt::SetIssuedAt(const QDateTime &DateTime) {
   d->IssuedAt = DateTime;
}
const QDateTime &TJwt::IssuedAt() const {
   return d->IssuedAt;
}
void TJwt::SetExpiration(const QDateTime &DateTime) {
   d->Expiration = DateTime;
}
const QDateTime &TJwt::Expiration() const {
   return d->Expiration;
}
void TJwt::SetAudience(const QString &Audience) {
   d->Audience = Audience;
}
const QString &TJwt::Audience() {
   return d->Audience;
}

namespace {
   // template <class TDetails>
   // struct TAlgoHandler {
   //     static const QString Name;
   // };
   // template <class TDetails>
   // const QString TAlgoHandler<TDetails>::Name = TDetails::Name;

   template <TJwt::TAlgo>
   struct TAlgoDetails;

   template <>
   struct TAlgoDetails<TJwt::ES256> {
      static const QString Name;
      static QByteArray    Hash(const QByteArray &StringToSign, const QByteArray &Secret) {
         return CalculateSignature(Secret, StringToSign).Data;
      }
   };
   const QString TAlgoDetails<TJwt::ES256>::Name = "ES256";

   // template <>
   // struct TAlgoDetails<TJwt::RS256> {
   //    static const QString Name;
   //    static QString       Hash(const QString &StringToSign, const QString &Secret) {}
   // };
   // const QString TAlgoDetails<TJwt::RS256>::Name = "RS256";

   // --------------------------------------------------------------------------------------------------

   QString ToString(const TJwt::TAlgo Algo) {
      switch (Algo) {
         // case TJwt::RS256: return TAlgoDetails<TJwt::RS256>::Name;
         case TJwt::ES256: return TAlgoDetails<TJwt::ES256>::Name;
      }
   }
   QString JsonObjectToString(const QJsonObject &Object) {
      // QString Result = QJsonDocument(Object).toJson(QJsonDocument::JsonFormat::Indented);
      QString Result = QJsonDocument(Object).toJson(QJsonDocument::Compact);
      return Result;
   }
   QString ComposeHeader(TJwt::TAlgo Algo) {
      return JsonObjectToString({{"alg", ToString(Algo)}, {"typ", "JWT"}});
   }
   QString ComposePayload(const TJwtPrivate &d) {
      QJsonObject Payload;
      if (d.IssuedAt.isValid())
         Payload["iat"] = d.IssuedAt.toSecsSinceEpoch();
      if (d.Expiration.isValid())
         Payload["exp	"] = d.Expiration.toSecsSinceEpoch();
      if (d.Audience.isEmpty() == false)
         Payload["aud"] = d.Audience;
      return JsonObjectToString(Payload);
   }
   QByteArray ComposeSignature(const TJwt::TAlgo Algo, const QByteArray &StringToSign, QIODevice &Secret) {
      switch (Algo) {
         // case TJwt::RS256: return TAlgoDetails<TJwt::RS256>::Hash(StringToSign, Secret.readAll());
         case TJwt::ES256: return TAlgoDetails<TJwt::ES256>::Hash(StringToSign, Secret.readAll());
      }
   }
   const QByteArray::Base64Options Base64Options = QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals;
} // namespace
QString TJwt::ComposeToken(QIODevice &Secret) const {
   const QString StrJsonHeader  = ComposeHeader(d->Algo);
   const QString StrJsonPayload = ComposePayload(*d);
   qDebug().noquote() << StrJsonHeader;
   qDebug().noquote() << StrJsonPayload;

   const QByteArray Base64Header  = QByteArray(StrJsonHeader.toLatin1()).toBase64(Base64Options);
   const QByteArray Base64Payload = QByteArray(StrJsonPayload.toLatin1()).toBase64(Base64Options);
   qDebug().noquote() << Base64Header;
   qDebug().noquote() << Base64Payload;


   QByteArray       Result          = Base64Header + "." + Base64Payload;
   const QByteArray Base64Signature = ComposeSignature(d->Algo, Result, Secret).toBase64(Base64Options);
   Result += "." + Base64Signature;
   return Result;
}
