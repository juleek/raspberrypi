#include "TJwt.h"

#include "TJwt_p.h"
#include "memory.h"

#include <QIODevice>
#include <QJsonDocument>
#include <QJsonObject>
#include <QtDebug>
#include <cstring>
#include <memory>
#include <openssl/bio.h>
#include <openssl/crypto.h>
#include <openssl/err.h>
#include <openssl/hmac.h>
#include <openssl/pem.h>
#include <openssl/sha.h>



QDebug operator<<(QDebug Out, const THashData &Bits) {
   Out << Bits.ToBase64Url();
   return Out;
}   // namespace

struct TOpenSSLDeleter {
   void operator()(EVP_MD_CTX *Ctx) {
      EVP_MD_CTX_destroy(Ctx);
   }
   void operator()(EVP_PKEY *Key) {
      EVP_PKEY_free(Key);
   }
   void operator()(BIO *Bio) {
      BIO_free(Bio);
   }
   void operator()(EVP_PKEY_CTX *Ctx) {
      EVP_PKEY_CTX_free(Ctx);
   }
   //   void operator()(ECDSA_SIG *Signature) {
   //      ECDSA_SIG_free(Signature);
   //   }
};
class TDigestCalculatorPrivate {
public:
   std::unique_ptr<EVP_MD_CTX, TOpenSSLDeleter> Ctx;
   bool                                         Ok = true;
};



// ===========================================================================================================
// Digest Calculator

namespace {
   const EVP_MD *AlgoToOpenSSL(const TDigestAlgo Algo) {
      switch(Algo) {
         case TDigestAlgo::SHA256: return EVP_sha256();
      }
   }
}   // namespace

TDigestCalculator::TDigestCalculator(const TDigestAlgo Algo): d(new TDigestCalculatorPrivate) {
   d->Ctx = UniquePtr(EVP_MD_CTX_create());
   if(d->Ctx == nullptr) {
      Q_ASSERT(false);
      d->Ok = false;
      return;
   }

   d->Ok = EVP_DigestInit(d->Ctx.get(), AlgoToOpenSSL(Algo));
   if(d->Ok == false) {
      Q_ASSERT(false);
      return;
   }
}
TDigestCalculator::~TDigestCalculator() = default;

TDigestCalculator &TDigestCalculator::AddData(const char *Data, size_t n) & {
   if(d->Ok == false) {
      Q_ASSERT(false);
      return *this;
   }

   d->Ok = EVP_DigestUpdate(d->Ctx.get(), Data, n);
   Q_ASSERT(d->Ok);

   return *this;
}
TDigestCalculator &TDigestCalculator::AddData(const QByteArray &Bytes) & {
   return AddData(Bytes.data(), Bytes.size());
}
TDigestCalculator &TDigestCalculator::AddData(QIODevice &Stream) & {
   static const constexpr size_t BUFFER_SIZE = 1 * 1024 * 1024;
   QByteArray                    buffer;
   buffer.resize(BUFFER_SIZE);

   for(; d->Ok;) {
      const size_t Actual = Stream.read(const_cast<char *>(buffer.data()), buffer.size());
      if(Actual == 0)
         break;
      AddData(buffer.data(), Actual);
   }
   Q_ASSERT(d->Ok);
   return *this;
}

TDigestCalculator &&TDigestCalculator::AddData(const char *Data, size_t n) && {
   return std::move(AddData(Data, n));
}
TDigestCalculator &&TDigestCalculator::AddData(const QByteArray &String) && {
   return std::move(AddData(String));
}
TDigestCalculator &&TDigestCalculator::AddData(QIODevice &Stream) && {
   return std::move(AddData(Stream));
}




// ===========================================================================================================

THashData CalculateSignature(TDigestCalculator &&Signer, const QByteArray &PrivateKey, const TKeyType KeyType) {
   TDigestCalculatorPrivate *d = Signer.d.get();

   auto OnError = [d](const int Line) {
      const uint64_t ErrorCode = ERR_peek_last_error();
      char           Buf[256]  = {};
      ERR_error_string_n(ErrorCode, Buf, 256);
      qDebug() << "Error at:" << Line << ":" << Buf;
      Q_ASSERT(false);
      d->Ok = false;
   };


   /// 1. Get digest (this what will be signed)

   if(d->Ok == false) {
      Q_ASSERT(false);
      return {};
   }

   QByteArray Digest;
   Digest.resize(EVP_MAX_MD_SIZE);
   unsigned int Len;
   const int    DigestOk = EVP_DigestFinal(d->Ctx.get(), reinterpret_cast<unsigned char *>(Digest.data()), &Len);
   if(DigestOk <= 0) {
      OnError(__builtin_LINE());
      return {};
   }
   Digest.resize(Len);
   // qDebug() << "Digest:" << Digest;


   /// 2. Open private key

   std::unique_ptr<BIO, TOpenSSLDeleter> Bio = UniquePtr(BIO_new_mem_buf(PrivateKey.data(), PrivateKey.size()));
   if(Bio == nullptr) {
      OnError(__builtin_LINE());
      return {};
   }

   // https://www.openssl.org/docs/man1.1.1/man3/PEM_read_bio_PrivateKey.html

   std::unique_ptr<EVP_PKEY, TOpenSSLDeleter> Key = UniquePtr(PEM_read_bio_PrivateKey(Bio.get(), nullptr, nullptr, nullptr));
   if(Key == nullptr) {
      OnError(__builtin_LINE());
      return {};
   }
   const int TypeOfKey = EVP_PKEY_id(Key.get());
   switch(KeyType) {
      case TKeyType::RSA: {
         if(TypeOfKey != EVP_PKEY_RSA) {
            OnError(__builtin_LINE());
            return {};
         }
         break;
      }
      case TKeyType::EC: {
         if(TypeOfKey != EVP_PKEY_EC) {
            OnError(__builtin_LINE());
            return {};
         }
         break;
      }
      case TKeyType::Any: break;
   }



   /// 3. Create ctx, initialise it, and sign the digest

   std::unique_ptr<EVP_PKEY_CTX, TOpenSSLDeleter> Ctx = UniquePtr(EVP_PKEY_CTX_new(Key.get(), nullptr));
   if(Ctx == nullptr) {
      OnError(__builtin_LINE());
      return {};
   }



   // Copied from https://beta.openssl.org/docs/man1.1.1/man3/EVP_PKEY_sign.html
   const int InitOk = EVP_PKEY_sign_init(Ctx.get());
   if(InitOk <= 0) {
      OnError(__builtin_LINE());
      return {};
   }
   const int PaddingSet = EVP_PKEY_CTX_set_rsa_padding(Ctx.get(), RSA_PKCS1_PADDING);
   if(PaddingSet <= 0) {
      OnError(__builtin_LINE());
      return {};
   }
   const int SignatureSet = EVP_PKEY_CTX_set_signature_md(Ctx.get(), EVP_sha256());
   if(SignatureSet <= 0) {
      OnError(__builtin_LINE());
      return {};
   }



   // Call EVP_PKEY_sign first time to get output buffer length
   size_t    SigLen;
   const int SignRes1 = EVP_PKEY_sign(Ctx.get(), nullptr, &SigLen, (const unsigned char *)Digest.constData(), Digest.size());
   if(SignRes1 <= 0) {
      OnError(__builtin_LINE());
      return {};
   }
   // Finally sign:
   THashData Result;
   Result.Data.resize(SigLen);
   const int SignRes2 = EVP_PKEY_sign(
       Ctx.get(), (unsigned char *)Result.Data.data(), &SigLen, (const unsigned char *)Digest.constData(), Digest.size());
   if(SignRes2 <= 0) {
      OnError(__builtin_LINE());
      return {};
   }

   // qDebug() << "Signature:" << Result.Data;

   return Result;
}

THashData
    CalculateSignature(const QByteArray &PrivateKey, const QByteArray &String, const TKeyType KeyType, const TDigestAlgo Algo) {
   return CalculateSignature(TDigestCalculator(Algo).AddData(String), PrivateKey, KeyType);
}
THashData CalculateSignature(const QByteArray &PrivateKey, QIODevice &Stream, const TKeyType KeyType, const TDigestAlgo Algo) {
   return CalculateSignature(TDigestCalculator(Algo).AddData(Stream), PrivateKey, KeyType);
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
   if(this == &Another)
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
void TJwt::SetTargetAudience(const QString &TargetAudience) {
   d->TargetAudience = TargetAudience;
}
const QString TJwt::TargetAudience() {
   return d->TargetAudience;
}
void TJwt::SetIss(const QString &Iss) {
   d->Iss = Iss;
}
const QString TJwt::Iss() {
   return d->Iss;
}
void TJwt::SetSub(const QString &Sub) {
   d->Sub = Sub;
}
const QString TJwt::Sub() {
   return d->Sub;
}




namespace {
   QString ToString(const TJwt::TAlgo Algo) {
      switch(Algo) {
         case TJwt::ES256: return "ES256";
         case TJwt::RS256: return "RS256";
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
      if(d.IssuedAt.isValid())
         Payload["iat"] = d.IssuedAt.toMSecsSinceEpoch() / 1000;
      if(d.Expiration.isValid())
         Payload["exp"] = d.Expiration.toMSecsSinceEpoch() / 1000;
      if(d.Audience.isEmpty() == false)
         Payload["aud"] = d.Audience;
      if(d.TargetAudience.isEmpty() == false)
         Payload["target_audience"] = d.TargetAudience;
      if(d.Sub.isEmpty() == false)
         Payload["sub"] = d.Sub;
      if(d.Iss.isEmpty() == false)
         Payload["iss"] = d.Iss;
      return JsonObjectToString(Payload);
   }
   THashData ComposeSignature(const TJwt::TAlgo Algo, const QByteArray &StringToSign, QIODevice &Secret) {
      // qDebug() << "Calculating signature of:" << StringToSign;
      switch(Algo) {
         case TJwt::ES256: return CalculateSignature(Secret.readAll(), StringToSign, TKeyType::EC, TDigestAlgo::SHA256);
         case TJwt::RS256: return CalculateSignature(Secret.readAll(), StringToSign, TKeyType::RSA, TDigestAlgo::SHA256);
      }
   }
   const QByteArray::Base64Options Base64Options = QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals;
}   // namespace

QString TJwt::ComposeToken(QIODevice &Secret) const {
   const QString StrJsonHeader  = ComposeHeader(d->Algo);
   const QString StrJsonPayload = ComposePayload(*d);
   // qDebug().noquote() << "JWT Header:" << StrJsonHeader;
   // qDebug().noquote() << "JWT Payload: " << StrJsonPayload;

   const QByteArray Base64Header  = QByteArray(StrJsonHeader.toLatin1()).toBase64(Base64Options);
   const QByteArray Base64Payload = QByteArray(StrJsonPayload.toLatin1()).toBase64(Base64Options);
   // qDebug().noquote() << Base64Header;
   // qDebug().noquote() << Base64Payload;

   QByteArray       Result          = Base64Header + "." + Base64Payload;
   const QByteArray Base64Signature = ComposeSignature(d->Algo, Result, Secret).Data.toBase64(Base64Options);
   Result += "." + Base64Signature;
   return Result;
}
