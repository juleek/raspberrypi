#include "../TJwt.h"

#include <QBuffer>
#include <QDebug>
#include <QFile>
#include <gtest/gtest.h>

TEST(TJWT, ComposeToken) {
   const QString FunctionName = "https://gf-thermo-vpd56udh7q-nw.a.run.app";
   const QString AccountEmail = "thermo-app-test-acc@tarasovka.iam.gserviceaccount.com";

   // https://cloud.google.com/functions/docs/securing/authenticating#exchanging_a_self-signed_jwt_for_a_google-signed_id_token
   TJwt Jwt;
   Jwt.SetAlgo(TJwt::RS256);
   Jwt.SetAudience("https://www.googleapis.com/oauth2/v4/token");
   Jwt.SetTargetAudience(FunctionName);
   Jwt.SetIss(AccountEmail);
   Jwt.SetSub(AccountEmail);
   Jwt.SetIssuedAt(QDateTime::currentDateTimeUtc());
   Jwt.SetExpiration(Jwt.IssuedAt().addSecs(60 * 60));

   QByteArray PrivateKey = R"_HERE_DOC_(
-----BEGIN PRIVATE KEY-----
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
V6Szl1N2KN65gJchtUTv6T0D2KsKlQ==
-----END PRIVATE KEY-----
)_HERE_DOC_";
   QBuffer    KeyStream  = {&PrivateKey};
   KeyStream.open(QBuffer::ReadOnly);


   const QString Result = Jwt.ComposeToken(KeyStream);
   qDebug() << "TJWT.ComposeToken: Signed JWT token:" << Result;
   // Verify it at https://jwt.io/
}
