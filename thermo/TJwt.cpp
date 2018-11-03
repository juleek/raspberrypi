#include "TJwt.h"
#include "TJwt_p.h"
#include "MakeUnique.h"

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


QDebug operator<<(QDebug Out, const THashData &Bits) {
   Out << Bits.ToBase64Url();
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
   void operator()(BIO *Bio) {
      BIO_free(Bio);
   }
   void operator()(ECDSA_SIG *Signature) {
      ECDSA_SIG_free(Signature);
   }
};
class TDigestSignerPrivate {
public:
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



TDigestSigner::TDigestSigner(const TDigestAlgo Algo)
    : d(new TDigestSignerPrivate) {
   d->Ctx = MakeUnique(EVP_MD_CTX_create());
   if (d->Ctx == nullptr) {
      Q_ASSERT(false);
      d->Ok = false;
      return;
   }

   d->Ok = EVP_DigestInit(d->Ctx.get(), AlgoToOpenSSL(Algo));
   if (d->Ok == false) {
      Q_ASSERT(false);
      return;
   }
}
TDigestSigner::~TDigestSigner() = default;

TDigestSigner &TDigestSigner::AddData(const char *Data, size_t n) & {
   if (d->Ok == false) {
      Q_ASSERT(false);
      return *this;
   }

   d->Ok = EVP_DigestUpdate(d->Ctx.get(), Data, n);
   Q_ASSERT(d->Ok);

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
   Q_ASSERT(d->Ok);
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

THashData CalculateSignature(TDigestSigner &&Signer, const QByteArray &PrivateKey) {
   TDigestSignerPrivate *d = Signer.d.get();
   if (d->Ok == false) {
      Q_ASSERT(false);
      return {};
   }

   QByteArray Digest;
   Digest.resize(EVP_MAX_MD_SIZE);
   unsigned int Len;
   d->Ok = EVP_DigestFinal(d->Ctx.get(), reinterpret_cast<unsigned char *>(Digest.data()), &Len);
   Digest.resize(Len);



   std::unique_ptr<BIO, TOpenSSLDeleter> Bio = MakeUnique(BIO_new_mem_buf(PrivateKey.data(), PrivateKey.size()));
   if (Bio == nullptr) {
      Q_ASSERT(false);
      d->Ok = false;
      return {};
   }

   std::unique_ptr<EC_KEY, TOpenSSLDeleter> ECKey =
       MakeUnique(PEM_read_bio_ECPrivateKey(Bio.get(), nullptr, nullptr, nullptr));
   if (ECKey == nullptr) {
      Q_ASSERT(false);
      d->Ok = false;
      return {};
   }

   std::unique_ptr<ECDSA_SIG, TOpenSSLDeleter> Signature =
       MakeUnique(ECDSA_do_sign(reinterpret_cast<unsigned char *>(Digest.data()), Digest.size(), ECKey.get()));
   if (Signature == nullptr) {
      Q_ASSERT(false);
      d->Ok = false;
      return {};
   }


   const BIGNUM *r;
   const BIGNUM *s;
   ECDSA_SIG_get0(Signature.get(), &r, &s);
   THashData Result;
   Result.Data.resize(64);
   BN_bn2bin(r, reinterpret_cast<unsigned char *>(Result.Data.data()));
   BN_bn2bin(s, reinterpret_cast<unsigned char *>(Result.Data.data()) + 32);
   // qDebug() << Result;
   return Result;

   // ECDSA_SIG *sig;
   // BIGNUM *r = BN_bin2bn(reinterpret_cast<const unsigned char *>(Result.Data.data()) + 4, 32, nullptr); // create new bn
   // here BIGNUM *s = BN_bin2bn(reinterpret_cast<const unsigned char *>(Result.Data.data()) + 4 + 32 + 2, 32, nullptr);
   // QByteArray ba;
   // ba.resize(64);
   // BN_bn2bin(r, reinterpret_cast<unsigned char *>(ba.data()));
   // BN_bn2bin(s, reinterpret_cast<unsigned char *>(ba.data()) + 32);
   // qDebug() << ba.toBase64(QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals);
   // BN_copy(sig->r, r);
   // BN_copy(sig->s, s);
}

THashData CalculateSignature(const QByteArray &PrivateKey, const QByteArray &String, TDigestAlgo Algo) {
   return CalculateSignature(TDigestSigner(Algo).AddData(String), PrivateKey);
}
THashData CalculateSignature(const QByteArray &PrivateKey, QIODevice &Stream, TDigestAlgo Algo) {
   return CalculateSignature(TDigestSigner(Algo).AddData(Stream), PrivateKey);
}

// openssl dgst -sha256 -sign ec_private.pem tosign.txt > signature.bin
// openssl dgst -sha256  -verify ec_public.pem -signature signature.bin tosign.txt
// Verified OK



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
   template <TJwt::TAlgo>
   struct TAlgoDetails;
   template <>
   struct TAlgoDetails<TJwt::ES256> {
      static const QString Name;
   };
   const QString TAlgoDetails<TJwt::ES256>::Name = "ES256";

   // --------------------------------------------------------------------------------------------------

   QString ToString(const TJwt::TAlgo Algo) {
      switch (Algo) {
         case TJwt::ES256: return TAlgoDetails<TJwt::ES256>::Name;
      }
   }
   QString JsonObjectToString(const QJsonObject &Object) {
      QString Result = QJsonDocument(Object).toJson(QJsonDocument::Compact);
      return Result;
   }
   QString ComposeHeader(TJwt::TAlgo Algo) {
      return JsonObjectToString({{"alg", ToString(Algo)}, {"typ", "JWT"}});
   }
   QString ComposePayload(const TJwtPrivate &d) {
      QJsonObject Payload;
      if (d.IssuedAt.isValid())
         Payload["iat"] = d.IssuedAt.toMSecsSinceEpoch() / 1000;
      if (d.Expiration.isValid())
         Payload["exp"] = d.Expiration.toMSecsSinceEpoch() / 1000;
      if (d.Audience.isEmpty() == false)
         Payload["aud"] = d.Audience;
      return JsonObjectToString(Payload);
   }
   THashData ComposeSignature(const TJwt::TAlgo Algo, const QByteArray &StringToSign, QIODevice &Secret) {
      switch (Algo) {
         case TJwt::ES256: return CalculateSignature(Secret.readAll(), StringToSign);
      }
   }
   const QByteArray::Base64Options Base64Options = QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals;
} // namespace

QString TJwt::ComposeToken(QIODevice &Secret) const {
   const QString StrJsonHeader  = ComposeHeader(d->Algo);
   const QString StrJsonPayload = ComposePayload(*d);
   // qDebug().noquote() << StrJsonHeader;
   // qDebug().noquote() << StrJsonPayload;

   const QByteArray Base64Header  = QByteArray(StrJsonHeader.toLatin1()).toBase64(Base64Options);
   const QByteArray Base64Payload = QByteArray(StrJsonPayload.toLatin1()).toBase64(Base64Options);
   // qDebug().noquote() << Base64Header;
   // qDebug().noquote() << Base64Payload;

   QByteArray       Result          = Base64Header + "." + Base64Payload;
   const QByteArray Base64Signature = ComposeSignature(d->Algo, Result, Secret).Data.toBase64(Base64Options);
   Result += "." + Base64Signature;
   return Result;
}
