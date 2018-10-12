#include "TDriver.h"

#include "TSmsSender.h"

#include <QCommandLineParser>
#include <QCoreApplication>
#include <QtDebug>
#include <QtMqtt/QMqttClient>

#include <memory>

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
   QCoreApplication app(argc, argv);

   std::unique_ptr<QMqttClient> MqttClient = std::make_unique<QMqttClient>();
   MqttClient->setHostname(MQTT_HOST);
   MqttClient->setClientId(CLIENT_ID);
   MqttClient->setUsername(MQTT_USERNAME);
   MqttClient->setPassword(CalculatePassword({}));

   MqttClient->connectToHostEncrypted(MQTT_HOST);

   QObject::connect(MqttClient.get(), &QMqttClient::connected, [&MqttClient]() { OnConnected(*MqttClient); });
   QObject::connect(MqttClient.get(),
                    &QMqttClient::messageReceived,
                    [](const QByteArray &Message, const QMqttTopicName &Topic) { OnMessageReceived(Message, Topic); });

   return app.exec();


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
