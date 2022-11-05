#include "../TJwtUpdater.h"

#include <QUrl>
#include <gtest/gtest.h>

TEST(TJWTUpdater_FormUrlEncode, ExpectedCase) {
   const QString Body = FormUrlEncode({{"grant_type", "urn:ietf:params:oauth:grant-type:jwt bearer"}, {"assertion", "asdf"}});
   ASSERT_EQ(Body, "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt+bearer&assertion=asdf");
}


TEST(TJWTUpdater_ParseIdTokenFromJson, ExpectedCase) {
   const QByteArray HttpBody = "{\"id_token\":\"eyJhbGciOiJSUzI1NiIsImtpZCI6ImY0NTEzNDVmYWQwODEwMWJmYjM0NWNmNjQyYTJkYTkyNjdiOWViZWIiLCJ0eXAiOiJKV1QifQ.eyJhdWQiOiJodHRwczovL2V1cm9wZS13ZXN0Mi10YXJhc292a2EuY2xvdWRmdW5jdGlvbnMubmV0L2dmLXRoZXJtby1nZW4xIiwiYXpwIjoidGhlcm1vLWFwcC10ZXN0LWFjY0B0YXJhc292a2EuaWFtLmdzZXJ2aWNlYWNjb3VudC5jb20iLCJlbWFpbCI6InRoZXJtby1hcHAtdGVzdC1hY2NAdGFyYXNvdmthLmlhbS5nc2VydmljZWFjY291bnQuY29tIiwiZW1haWxfdmVyaWZpZWQiOnRydWUsImV4cCI6MTY2NzY2Mjg5OSwiaWF0IjoxNjY3NjU5Mjk5LCJpc3MiOiJodHRwczovL2FjY291bnRzLmdvb2dsZS5jb20iLCJzdWIiOiIxMTI2MDQ4MTg2Nzk1OTQyNzI0MTIifQ.hkP5IfqYfcCHlJMqsguiJDhBd7oYHW4uSVXjtzVx0ehKGOqVQt96luZFadmKar3hOD94iNbiZMrklukjck_wCBOr_uPBKpuGaCex37WIs9oD6Abru-80Wh1o0S-EtL8iy1AfAY5QGd-buaePT2Ed8FE3zxqozEx2PhRgckQ0me7rJAUsMoVylIzP5lvcDt3-Pac1U0B9z2W_iu_-RDfoBUa8zoX611xMUyf9TgRSSDpZi0LHYFPgxgBDl7kx28-hXc3qplHxyKOdpOboANwDvmx-yUrrsvpzsiqGVP4PhxQ3Jz-92z7i0CgMm8TTxu3Ym2qJrmG1k4knMa8flQxweg\"}";
   const QString Expected = "eyJhbGciOiJSUzI1NiIsImtpZCI6ImY0NTEzNDVmYWQwODEwMWJmYjM0NWNmNjQyYTJkYTkyNjdiOWViZWIiLCJ0eXAiOiJKV1QifQ.eyJhdWQiOiJodHRwczovL2V1cm9wZS13ZXN0Mi10YXJhc292a2EuY2xvdWRmdW5jdGlvbnMubmV0L2dmLXRoZXJtby1nZW4xIiwiYXpwIjoidGhlcm1vLWFwcC10ZXN0LWFjY0B0YXJhc292a2EuaWFtLmdzZXJ2aWNlYWNjb3VudC5jb20iLCJlbWFpbCI6InRoZXJtby1hcHAtdGVzdC1hY2NAdGFyYXNvdmthLmlhbS5nc2VydmljZWFjY291bnQuY29tIiwiZW1haWxfdmVyaWZpZWQiOnRydWUsImV4cCI6MTY2NzY2Mjg5OSwiaWF0IjoxNjY3NjU5Mjk5LCJpc3MiOiJodHRwczovL2FjY291bnRzLmdvb2dsZS5jb20iLCJzdWIiOiIxMTI2MDQ4MTg2Nzk1OTQyNzI0MTIifQ.hkP5IfqYfcCHlJMqsguiJDhBd7oYHW4uSVXjtzVx0ehKGOqVQt96luZFadmKar3hOD94iNbiZMrklukjck_wCBOr_uPBKpuGaCex37WIs9oD6Abru-80Wh1o0S-EtL8iy1AfAY5QGd-buaePT2Ed8FE3zxqozEx2PhRgckQ0me7rJAUsMoVylIzP5lvcDt3-Pac1U0B9z2W_iu_-RDfoBUa8zoX611xMUyf9TgRSSDpZi0LHYFPgxgBDl7kx28-hXc3qplHxyKOdpOboANwDvmx-yUrrsvpzsiqGVP4PhxQ3Jz-92z7i0CgMm8TTxu3Ym2qJrmG1k4knMa8flQxweg";
   const QString &Actual = ParseIdTokenFromJson(HttpBody);
   ASSERT_EQ(Actual, Expected);
}

TEST(TJWTUpdater_ParseIdTokenFromJson, Empty) {
   const QByteArray HttpBody = "";
   const QString Expected = "";
   const QString &Actual = ParseIdTokenFromJson(HttpBody);
   ASSERT_EQ(Actual, Expected);
}

TEST(TJWTUpdater_ParseIdTokenFromJson, NotAString) {
   const QByteArray HttpBody = "{\"id_token\":42}";
   const QString Expected = "";
   const QString &Actual = ParseIdTokenFromJson(HttpBody);
   ASSERT_EQ(Actual, Expected);
}

TEST(TJWTUpdater_ParseIdTokenFromJson, RootIsAnArray) {
   const QByteArray HttpBody = "[]";
   const QString Expected = "";
   const QString &Actual = ParseIdTokenFromJson(HttpBody);
   ASSERT_EQ(Actual, Expected);
}
