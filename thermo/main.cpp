#include "TDriver.h"
#include "TJwt.h"
#include "TSmsSender.h"

#include <QCommandLineParser>
#include <QCoreApplication>
#include <QDebug>
#include <QFile>
#include <QtMqtt/QMqttClient>
#include <memory>

//#include <openssl/bio.h>
//#include <openssl/conf.h>
//#include <openssl/crypto.h>
//#include <openssl/err.h>
//#include <openssl/hmac.h>
//#include <openssl/pem.h>
//#include <openssl/rand.h>
//#include <openssl/sha.h>
//#include <openssl/ssl.h>
// void OpenSSLTest() {
//    SSL_library_init();
//    SSL_load_error_strings();
//    OpenSSL_add_all_algorithms();
//    OpenSSL_add_all_ciphers();
//    OpenSSL_add_all_digests();
//    ERR_load_BIO_strings();
//
//    QByteArray dt         = "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJhc2RmIn0";
//    QFile      PrivateKey = {"/home/Void/devel/gc/ec_private.pem"};
//    PrivateKey.open(QIODevice::ReadOnly);
//    QByteArray pk = PrivateKey.readAll();
//
//
//    // EVP_MD_CTX *Ctx = EVP_MD_CTX_create();
//    // EVP_DigestInit(Ctx, EVP_sha256());
//    // EVP_DigestUpdate(Ctx, dt.data(), dt.size());
//    // QByteArray Digest;
//    // Digest.resize(EVP_MAX_MD_SIZE);
//    // unsigned int Len;
//    // EVP_DigestFinal(Ctx, reinterpret_cast<unsigned char *>(Digest.data()), &Len);
//    // Digest.resize(Len);
//    //
//    // BIO *   Bio   = BIO_new_mem_buf(pk.data(), pk.size());
//    // EC_KEY *ECKey = PEM_read_bio_ECPrivateKey(Bio, nullptr, nullptr, nullptr);
//    // ECDSA_SIG *Signature = ECDSA_do_sign(reinterpret_cast<unsigned char *>(Digest.data()), Digest.size(), ECKey);
//    //
//    // const BIGNUM *r;
//    // const BIGNUM *s;
//    // ECDSA_SIG_get0(Signature, &r, &s);
//    // QByteArray ba;
//    // ba.resize(64);
//    // BN_bn2bin(r, reinterpret_cast<unsigned char *>(ba.data()));
//    // BN_bn2bin(s, reinterpret_cast<unsigned char *>(ba.data()) + 32);
//    // qDebug() << ba.toBase64(QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals);
//
//
//
//
//
//    // EVP_MD_CTX *Ctx    = EVP_MD_CTX_create();
//    // BIO *       Bio    = BIO_new_mem_buf(pk.data(), pk.size());
//    // EVP_PKEY *  EVPKey = PEM_read_bio_PrivateKey(Bio, nullptr, nullptr, nullptr);
//    // EVP_DigestSignInit(Ctx, nullptr, EVP_sha256(), nullptr, EVPKey);
//    // QByteArray dt = "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJhc2RmIn0";
//    // EVP_DigestSignUpdate(Ctx, dt.data(), dt.size());
//    // QByteArray Result;
//    // Result.resize(1024 * 1024 * 1024);
//    // size_t Len;
//    // EVP_DigestSignFinal(Ctx, reinterpret_cast<unsigned char *>(Result.data()), &Len);
//    // Result.resize(Len);
//    // qDebug() << Result.toBase64(QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals);
//
//    // ECDSA_SIG *sig;
//    // BIGNUM *   r = BN_bin2bn(reinterpret_cast<const unsigned char *>(Result.data()) + 4, 32, nullptr); // create new bn
//    // here BIGNUM *   s = BN_bin2bn(reinterpret_cast<const unsigned char *>(Result.data()) + 4 + 32 + 2, 32, nullptr);
//    // QByteArray ba;
//    // ba.resize(64);
//    // BN_bn2bin(r, reinterpret_cast<unsigned char *>(ba.data()));
//    // BN_bn2bin(s, reinterpret_cast<unsigned char *>(ba.data()) + 32);
//    // qDebug() << ba.toBase64(QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals);
//    //
//    //
//    // QFile Signature = {"/home/Void/devel/gc/signature.bin"};
//    // Signature.open(QIODevice::WriteOnly);
//    // Signature.write(Result);
//
//
//    // const THashData HashData = CalculateSignature(PrivateKey.readAll(),
//    // "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJhc2RmIn0"); qDebug() <<
//    // HashData.Data.toBase64(QByteArray::Base64UrlEncoding | QByteArray::OmitTrailingEquals); QFile Signature =
//    // {"/home/Void/devel/gc/signature.bin"}; Signature.open(QIODevice::WriteOnly); Signature.write(HashData.Data);
// }


// void InProcTests(QString SmsPass) {
//   TSmsSender SmsSender("dimanne", SmsPass, "Tarasovka", { {0, {QTime(0, 0, 10)}} });
//   SmsSender.Send(0, "text of message", { {"+79647088442"} });
//   QThread::sleep(25);
//   SmsSender.Send(0, "123", { {"+79647088442"} });
//   QThread::sleep(25);
//   SmsSender.Send(0, "455", { {"+79647088442"} });
//   QThread::sleep(25);
//   SmsSender.Send(0, "678", { {"+79647088442"} });

//}

namespace {
   const QString MQTT_HOST      = "mqtt.googleapis.com"; // :8883
   const QString MQTT_USERNAME  = {};
   const QString DEVICE_ID_TEST = "device_test_imp";
   const QString DEVICE_ID_MAIN = "device_tarpi";

   const QString DEVICE_ID  = DEVICE_ID_TEST;
   const QString MQTT_TOPIC = "/devices/" + DEVICE_ID + "/events";
   const QString CLIENT_ID =
       "projects/tarasovka-monitoring/locations/europe-west1/registries/temperature/devices/" + DEVICE_ID;

   QString CalculatePassword(const QString &PrivateKey) {
      return {};
   }
} // namespace

void OnConnected(QMqttClient &MqttClient) {
   qDebug() << "Connected!";
   MqttClient.disconnectFromHost();
}
void OnMessageReceived(const QByteArray &Message, const QMqttTopicName &Topic) {
   qDebug() << QDateTime::currentDateTime().toString() << QLatin1String(" Received Topic: ") << Topic.name()
            << QLatin1String(" Message: ") << Message;
   Q_ASSERT(false);
}

int main(int argc, char **argv) {
   // OpenSSLTest();
   // ---------------------------------------------------------------------------------------------------------

   // QFile         PrivateKey = {"/home/Void/devel/gc/ec_private.pem"};
   // PrivateKey.open(QIODevice::ReadOnly);
   // TDigestSigner Signer(TDigestAlgo::SHA256);
   // Signer.AddData("eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJhc2RmIn0");
   // THashData Signature = CalculateSignature(std::move(Signer), PrivateKey.readAll());
   // qDebug() << Signature;
   // return 0;

   // ---------------------------------------------------------------------------------------------------------

   QFile PrivateKey = {"/home/Void/devel/gc/ec_private.pem"};
   PrivateKey.open(QIODevice::ReadOnly);
   TJwt Jwt;
   Jwt.SetAudience("asdf");
   const QString Token = Jwt.ComposeToken(PrivateKey);
   qDebug() << Token;

   // ---------------------------------------------------------------------------------------------------------


   // QCoreApplication app(argc, argv);
   //
   // std::unique_ptr<QMqttClient> MqttClient = std::make_unique<QMqttClient>();
   // MqttClient->setHostname(MQTT_HOST);
   // MqttClient->setClientId(CLIENT_ID);
   // MqttClient->setUsername(MQTT_USERNAME);
   // MqttClient->setPassword(CalculatePassword({}));
   //
   // MqttClient->connectToHostEncrypted(MQTT_HOST);
   //
   // QObject::connect(MqttClient.get(), &QMqttClient::connected, [&MqttClient]() { OnConnected(*MqttClient); });
   // QObject::connect(MqttClient.get(),
   //                  &QMqttClient::messageReceived,
   //                  [](const QByteArray &Message, const QMqttTopicName &Topic) { OnMessageReceived(Message, Topic); });
   //
   // return app.exec();


   // ---------------------------------------------------------------------------------------------------------


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
