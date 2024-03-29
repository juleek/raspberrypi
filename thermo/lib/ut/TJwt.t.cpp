#include "../TJwt.h"

#include <QBuffer>
#include <QDebug>
#include <QFile>
#include <gtest/gtest.h>

TEST(TJWT, ComposeToken) {
   // gcloud functions deploy gf-thermo-gen1 --region=europe-west2 --trigger-http --runtime=python310
   // --source=/home/dimanne/devel/raspberrypi/thermo/test --entry-point=hello_http
   // const QString FunctionName = "https://gf-thermo-vpd56udh7q-nw.a.run.app";
   const QString FunctionName = "https://europe-west2-tarasovka.cloudfunctions.net/gf-thermo-gen1";
   const QString AccountEmail = "thermo-app-test-acc@tarasovka.iam.gserviceaccount.com";

   // https://cloud.google.com/functions/docs/securing/authenticating#exchanging_a_self-signed_jwt_for_a_google-signed_id_token
   TJwt Jwt = {
       .Algo           = TJwt::RS256,
       .Audience       = "https://www.googleapis.com/oauth2/v4/token",
       .TargetAudience = FunctionName,
       .Sub            = AccountEmail,
       .Iss            = AccountEmail,
       // .Scopes         = "https://www.googleapis.com/auth/cloud-platform"
   };

   QByteArray PrivateKey = R"_HERE_DOC_(
-----BEGIN PRIVATE KEY-----
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
YNu8Dg8vxh7u6a3QgRFV5arxpRNE
-----END PRIVATE KEY-----
)_HERE_DOC_";
   QBuffer    KeyStream  = {&PrivateKey};
   KeyStream.open(QBuffer::ReadOnly);


   const QString Result = Jwt.ComposeSignedToken(KeyStream);
   qDebug() << "TJWT.ComposeToken: Signed JWT token:" << Result;
   // Verify it at https://jwt.io/
}
