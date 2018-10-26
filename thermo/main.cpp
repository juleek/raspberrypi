#include <QCommandLineParser>
#include <QCoreApplication>
#include <QDebug>

#include "TGCMqtt.h"

int InlineTest(int argc, char **argv) {
   // OpenSSLTest();
   // ---------------------------------------------------------------------------------------------------------

   // QFile         PrivateKey = {"/home/Void/devel/gc/ec_private.pem"};
   // PrivateKey.open(QIODevice::ReadOnly);
   // TDigestSigner Signer(TDigestAlgo::SHA256);
   // Signer.AddData("eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJhc2RmIn0");
   // THashData Signature = CalculateSignature(std::move(Signer),
   // PrivateKey.readAll()); qDebug() << Signature; return 0;

   // ---------------------------------------------------------------------------------------------------------

   // QFile PrivateKey = {"/home/Void/devel/gc/ec_private.pem"};
   // PrivateKey.open(QIODevice::ReadOnly);
   // TJwt Jwt;
   // Jwt.SetAudience("asdf");
   // const QString Token = Jwt.ComposeToken(PrivateKey);
   // qDebug() << Token;

   // ---------------------------------------------------------------------------------------------------------

   QCoreApplication app(argc, argv);

   static const QString DEVICE_ID_TEST = "device_test_imp";
   static const QString DEVICE_ID_MAIN = "device_tarpi";

   TGCMqttSetup MqttSetup;
   MqttSetup.ProjectId      = "tarasovka-monitoring";
   MqttSetup.RegistryId     = "temperature";
   MqttSetup.DeviceId       = DEVICE_ID_TEST;
   MqttSetup.PrivateKeyPath = "/home/Void/devel/gc/ec_private.pem";

   TGCMqtt GCMqtt(MqttSetup);
   GCMqtt.Publish(12, 29);

   return app.exec();

   // ---------------------------------------------------------------------------------------------------------
}

int main(int argc, char **argv) {
   return InlineTest(argc, argv);

   // QCoreApplication   app(argc, argv);
   // QCommandLineParser Parser;
   //
   // QCommandLineOption SMSPassOpt = QCommandLineOption("SMSPass", "Password for SMS gate", "String");
   // Parser.addOption(SMSPassOpt);
   // Parser.process(app);
   //
   // QString SMSPass = Parser.value(SMSPassOpt);
   // qDebug() << "SMSPass:" << SMSPass;
   //
   // // InProcTests(SMSPass);
   //
   // const std::vector<TSensorInfo> SensorInfos = {{"/sys/bus/w1/devices/28-000005eac50a/w1_slave", "BottomTube", 12},
   //                                               {"/sys/bus/w1/devices/28-000005eaddc2/w1_slave", "Ambient", 6}};
   //
   // const QTime SendSMSStartTime = QTime(18, 15, 0);
   // const QTime SendSMSEndTime   = QTime(19, 30, 0);
   //
   // new TDriver(SMSPass, SensorInfos, SendSMSStartTime, SendSMSEndTime);
   // return app.exec();
}
