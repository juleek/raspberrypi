#include "../lib/THttpSink.h"
#include "../lib/TJwtUpdater.h"
#include "../lib/TSensorPoller.h"
#include "../lib/TSensorsPoller.h"
#include "../lib/memory.h"

#include <QCommandLineParser>
#include <QCoreApplication>
#include <QDebug>
#include <QFile>
#include <QNetworkAccessManager>
#include <QSocketNotifier>
#include <signal.h>
#include <sys/socket.h>
#include <unistd.h>


// ===========================================================================================================

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

   TJwtUpdater::TCfg Cfg = {.FuncHttpEndPoint = "https://europe-west2-tarasovka.cloudfunctions.net/gf-thermo-gen1",
                            .AccountEmail     = "thermo-app-test-acc@tarasovka.iam.gserviceaccount.com",
                            .PrivateKey       = R"_HERE_DOC_(
-----BEGIN PRIVATE KEY-----
-----END PRIVATE KEY-----
)_HERE_DOC_"};

   QNetworkAccessManager NetworkAccessManager;
   TJwtUpdater           Updater {std::move(Cfg), NetworkAccessManager};
   Updater.Start();
   return app.exec();

   // return 0;
}





// ===========================================================================================================

std::tuple<THttpSink::TCfg, TJwtUpdater::TCfg> CfgsFromCmdLineArgs(QCoreApplication &app) {
   QCommandLineOption GFPrivateKeyPathOption = {"GFPrivateKeyPath", "Path of the private key for Google Function", "String"};
   QCommandLineOption GFAccountEmailOption   = {"GFAccountEmail", "Service Account Email", "String"};
   QCommandLineOption GFHttpEndPoint         = {"GFHttpEndPoint", "Google Function Http end-point", "String"};
   QCommandLineOption GFDryRunOption         = {"DryRun", "If true we will not publish any data to Google Cloud"};
   QCommandLineParser Parser;
   Parser.addOption(GFPrivateKeyPathOption);
   Parser.addOption(GFAccountEmailOption);
   Parser.addOption(GFHttpEndPoint);
   Parser.addOption(GFDryRunOption);
   Parser.addHelpOption();
   Parser.addVersionOption();
   Parser.process(app);

   if(Parser.isSet(GFPrivateKeyPathOption) == false) {
      qDebug() << "There is no" << GFPrivateKeyPathOption.names() << " => exiting...";
      exit(1);
   }

   const QString &PrivateKeyFilename = Parser.value(GFPrivateKeyPathOption);
   QFile          PrivateKeyFile     = {PrivateKeyFilename};
   const bool     OpenedOk           = PrivateKeyFile.open(QIODevice::ReadOnly);
   if(OpenedOk == false) {
      qDebug() << "Failed to Open: " << PrivateKeyFilename << ":"
               << "error:" << PrivateKeyFile.errorString() << " => exiting...";
      exit(1);
   }
   const QByteArray PrivateKey = PrivateKeyFile.readAll();

   THttpSink::TCfg   HttpSinkCfg   = {.FuncHttpEndPoint = Parser.value(GFHttpEndPoint), .DryRun = Parser.isSet(GFDryRunOption)};
   TJwtUpdater::TCfg JwtUpdaterCfg = {.FuncHttpEndPoint = Parser.value(GFHttpEndPoint),
                                      .AccountEmail     = Parser.value(GFAccountEmailOption),
                                      .PrivateKey       = PrivateKey};


   return {HttpSinkCfg, JwtUpdaterCfg};
}


// -----------------------------------------------------------------------------------------------------------
// Signal handler (Ctrl+C)
namespace {
   std::unique_ptr<QSocketNotifier> SigIntSocketNotifier;
   int                              SigIntFd[2];

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

   SigIntSocketNotifier = MakeUnique(SigIntFd[1], QSocketNotifier::Read);
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

   auto [HttpSinkCfg, JwtUpdaterCfg] = CfgsFromCmdLineArgs(app);

   QNetworkAccessManager NetworkAccessManager;

   TJwtUpdater JwtUpdater {std::move(JwtUpdaterCfg), NetworkAccessManager};
   THttpSink   Sink {std::move(HttpSinkCfg), JwtUpdater, NetworkAccessManager};

   JwtUpdater.Start();

   const std::vector<TSensorInfo> SensorInfos = {{"/sys/bus/w1/devices/28-000005eac50a/w1_slave", "BottomTube"},
                                                 {"/sys/bus/w1/devices/28-000005eaddc2/w1_slave", "Ambient"}};
   TSensorsPoller                 SensorsPoller {SensorInfos, Sink};
   return app.exec();
}
