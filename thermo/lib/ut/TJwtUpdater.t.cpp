#include "../TJwtUpdater.h"

#include <QUrl>
#include <gtest/gtest.h>

TEST(TJWTUpdater, FormUrlEncode) {
   const QString Body = FormUrlEncode({{"grant_type", "urn:ietf:params:oauth:grant-type:jwt bearer"}, {"assertion", "asdf"}});
   ASSERT_EQ(Body, "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt+bearer&assertion=asdf");
}


