#include "TDriver.h"
#include "TGCMqtt.h"
#include "TTempPoller.h"
#include "MakeUnique.h"

#include <QCommandLineParser>
#include <QCoreApplication>
#include <QDebug>
#include <QSocketNotifier>
#include <sys/socket.h>
#include <unistd.h>
#include <signal.h>


namespace {
   const QString DEVICE_ID_TEST = "device_test_imp";
   const QString DEVICE_ID_MAIN = "device_tarpi";
}   // namespace




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

   TGCMqttSetup MqttSetup;
   MqttSetup.ProjectId      = "tarasovka-monitoring";
   MqttSetup.RegistryId     = "temperature";
   MqttSetup.DeviceId       = DEVICE_ID_TEST;
   MqttSetup.PrivateKeyPath = "/home/Void/devel/gc/ec_private.pem";

   TGCMqtt GCMqtt(MqttSetup);
   GCMqtt.Publish({{{"BottomTube", 12}, {"Ambient", 29}}, {}});

   return app.exec();

   // ---------------------------------------------------------------------------------------------------------
}

void HandleCommandLineOptions(QCoreApplication &app, TGCMqttSetup &MqttSetup) {
   QCommandLineOption MQTTPrivateKeyPathOption = {"MQTTPrivateKeyPath", "Path of the private key for MQTT", "String"};
   QCommandLineOption MQTTDryRunOption         = {"MQTTDryRun", "If true we will not publish any data to Google Cloud"};
   QCommandLineOption GCDeviceIdOption         = {"GCDeviceId", "Device id, as it registered in Google Cloud", "String"};
   QCommandLineParser Parser;
   Parser.addOption(MQTTPrivateKeyPathOption);
   Parser.addOption(MQTTDryRunOption);
   Parser.addOption(GCDeviceIdOption);
   Parser.addHelpOption();
   Parser.addVersionOption();
   Parser.process(app);

   if(Parser.isSet(MQTTPrivateKeyPathOption) == false) {
      qDebug() << "There is no" << MQTTPrivateKeyPathOption.names() << " => exiting...";
      exit(1);
   }
   MqttSetup.PrivateKeyPath = Parser.value(MQTTPrivateKeyPathOption);   // "/home/Void/devel/gc/ec_private.pem";

   if(Parser.isSet(GCDeviceIdOption))
      MqttSetup.DeviceId = Parser.value(GCDeviceIdOption);

   if(Parser.isSet(MQTTDryRunOption))
      MqttSetup.DryRun = true;
}


// -----------------------------------------------------------------------------------------------------------
// Signal handler (Ctrl+C)
namespace {
   std::unique_ptr<QSocketNotifier> SigIntSocketNotifier;
   int  SigIntFd[2];
   void UnixSignalHandler(int) {
      char a = 1;
      ::write(SigIntFd[0], &a, sizeof(a));
   }
}   // namespace
void OnSigInt() {
   SigIntSocketNotifier->setEnabled(false);
   qDebug() << "Signal INT received - exiting...";
   qApp->quit();
}
void InitSignalHandlers() noexcept {
   const int res = socketpair(AF_UNIX, SOCK_STREAM, 0, SigIntFd);
   if(res != 0)
      std::abort();

   SigIntSocketNotifier = MakeUnique(new QSocketNotifier(SigIntFd[1], QSocketNotifier::Read));
   QObject::connect(SigIntSocketNotifier.get(), &QSocketNotifier::activated, [](int) { OnSigInt(); });

   struct sigaction Int;
   Int.sa_handler = UnixSignalHandler;
   sigemptyset(&Int.sa_mask);
   Int.sa_flags = 0;
   if(sigaction(SIGINT, &Int, 0) > 0)
      std::abort();
}
// -----------------------------------------------------------------------------------------------------------


int main(int argc, char **argv) {
   // return InlineTest(argc, argv);

   QCoreApplication app(argc, argv);
   InitSignalHandlers();

   TGCMqttSetup MqttSetup;
   MqttSetup.ProjectId  = "tarasovka-monitoring";
   MqttSetup.RegistryId = "temperature";
   MqttSetup.DeviceId   = DEVICE_ID_TEST;

   HandleCommandLineOptions(app, MqttSetup);

   const std::vector<TSensorInfo> SensorInfos = {{"/sys/bus/w1/devices/28-000005eac50a/w1_slave", "BottomTube"},
                                                 {"/sys/bus/w1/devices/28-000005eaddc2/w1_slave", "Ambient"}};
   new TDriver(SensorInfos, MqttSetup);
   return app.exec();
}
