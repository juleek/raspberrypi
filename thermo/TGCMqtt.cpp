#include "TGCMqtt.h"
#include "TJwt.h"
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
// #include <QRandomGenerator>
#include <QTimer>
#include <QtMqtt/QMqttClient>
#include <experimental/optional>
#include <random>



QDebug operator<<(QDebug Out, const TGCMqttSetup &Setup) {
   Out << "============ TGCMqttSetup ============\n";
   Out << "Host:" << Setup.Host << "\n";
   Out << "Port:" << Setup.Port << "\n";
   Out << "UserName:" << Setup.UserName << "\n";
   Out << "Location:" << Setup.Location << "\n";
   Out << "DryRun:" << Setup.DryRun << "\n";
   Out << "ProjectId:" << Setup.ProjectId << "\n";
   Out << "DeviceId:" << Setup.DeviceId << "\n";
   Out << "RegistryId:" << Setup.RegistryId << "\n";
   Out << "PrivateKeyPath:" << Setup.PrivateKeyPath << "\n";

   Out << "====================================\n\n";
   return Out;
}


namespace {

   QByteArray TelemetryToJson(const TPublishItem &Item) {
      /// Json with the following fields is expected by google function:
      /// sensor_id_bottom_tube = "BottomTube";
      /// sensor_id_ambient = "Ambient";
      /// error_string_id = "ErrorString"

      QJsonObject Object;
      for (auto It = Item.NameToTemp.cbegin(); It != Item.NameToTemp.cend(); ++It) {
         const QString &SensorName  = It.key();
         const double   Temperature = It.value();
         Object[SensorName]         = Temperature;
      }
      if (Item.ErrorString.isEmpty() == false)
         Object["ErrorString"] = Item.ErrorString;
      QByteArray Result = QJsonDocument(Object).toJson(QJsonDocument::Compact);
      return Result;
   }

   QString CalculatePassword(const QString &PrivateKeyPath, const QString &ProjectId) {
      QFile PrivateKey = {PrivateKeyPath};
      PrivateKey.open(QIODevice::ReadOnly);

      TJwt Jwt;
      Jwt.SetIssuedAt(QDateTime::currentDateTimeUtc());
      Jwt.SetExpiration(Jwt.IssuedAt().addDays(1));
      Jwt.SetAudience(ProjectId);

      const QString Result = Jwt.ComposeToken(PrivateKey);
      // qDebug() << "Mqtt Password: " << Result;
      return Result;
   }
} // namespace

QString TGCMqttSetup::ClientId() const {
   return "projects/" + ProjectId + "/locations/" + Location + "/registries/" + RegistryId + "/devices/" + DeviceId;
}
QString TGCMqttSetup::Topic() const {
   return "/devices/" + DeviceId + "/events";
}

class TGCMqttPrivate {
public:
   TGCMqttSetup Setup;
   QMqttClient  Client;

   void Init();

   void OnMessageReceived(const QByteArray &Message, const QMqttTopicName &Topic);
   void OnDisconnected();
   void OnConnected();

   void                                      PublishIfNeeded();
   std::experimental::optional<TPublishItem> ItemToPublish;

   size_t               NumberOfFailedConnects = 0;
   size_t               GetBackoffDurationMSec() const;
   void                 ScheduleReConnect();
   std::random_device   rd;                       // obtain a random number from hardware
   mutable std::mt19937 eng = std::mt19937{rd()}; // seed the generator
};

void TGCMqttPrivate::Init() {
   Client.setProtocolVersion(QMqttClient::MQTT_3_1_1);
   Client.setHostname(Setup.Host);
   Client.setPort(Setup.Port);
   Client.setClientId(Setup.ClientId());
   Client.setUsername(Setup.UserName);

   QObject::connect(&Client, &QMqttClient::connected, [this]() { OnConnected(); });
   QObject::connect(&Client, &QMqttClient::disconnected, [this]() { OnDisconnected(); });
   QObject::connect(&Client, &QMqttClient::messageReceived, [this](const QByteArray &m, const QMqttTopicName &t) {
      OnMessageReceived(m, t);
   });

   ScheduleReConnect();
}
size_t TGCMqttPrivate::GetBackoffDurationMSec() const {
   // Exponential Backoff
   // https://cloud.google.com/iot/docs/how-tos/exponential-backoff

   static const unsigned long long MAX_MSECS = 20ull * 1000;
   static const size_t             MIN_MSECS = 1;

   if (NumberOfFailedConnects == 0)
      return MIN_MSECS;

   size_t Result = NumberOfFailedConnects > 10 ? MAX_MSECS : std::min(MAX_MSECS, (1ull << NumberOfFailedConnects) * 1000);
   // Result += QRandomGenerator::global()->bounded(static_cast<quint32>(Result / 2));
   Result += std::uniform_int_distribution<>(0, Result / 2)(eng); // define the range
   return Result;
}

void TGCMqttPrivate::ScheduleReConnect() {
   const auto ReConnect = [this]() {
      ++NumberOfFailedConnects;
      Client.setPassword(CalculatePassword(Setup.PrivateKeyPath, Setup.ProjectId));
      Client.connectToHostEncrypted(Setup.Host);
   };
   const int BackOffDuration = GetBackoffDurationMSec();
   qDebug() << "TGCMqtt:"
            << "ScheduleReConnect:"
            << "(Re-)Connection scheduled in" << BackOffDuration << "msec";
   QTimer::singleShot(BackOffDuration, ReConnect);
}

void TGCMqttPrivate::OnConnected() {
   qDebug() << "TGCMqtt:"
            << "OnConnected:" << Client.error() << Client.state();
   NumberOfFailedConnects = 0;
   PublishIfNeeded();
}
void TGCMqttPrivate::OnMessageReceived(const QByteArray &Message, const QMqttTopicName &Topic) {
   qDebug() << "TGCMqtt:"
            << "Received Topic:" << Topic.name() << "Message:" << Message;
   Q_ASSERT(false);
}
void TGCMqttPrivate::OnDisconnected() {
   qDebug() << "TGCMqtt:"
            << "OnDisconnected:" << Client.error() << Client.state();
   ScheduleReConnect();
}

void TGCMqttPrivate::PublishIfNeeded() {
   if (!ItemToPublish)
      return;
   if (Client.state() != QMqttClient::Connected)
      return;

   const QByteArray DataInJson = TelemetryToJson(*ItemToPublish);
   qDebug() << "TGCMqtt:"
            << "Publishing:" << DataInJson;
   if (Setup.DryRun == false)
      Client.publish(Setup.Topic(), DataInJson);
   ItemToPublish = std::experimental::optional<TPublishItem>();
}


// ----------------------------------------------------------------------------------------------------------------------


TGCMqtt::TGCMqtt(const TGCMqttSetup &Setup)
    : d(new TGCMqttPrivate) {
   qDebug() << Setup;
   d->Setup = Setup;
   d->Init();
}
TGCMqtt::~TGCMqtt() = default;

void TGCMqtt::Publish(const TPublishItem &PublishItem) {
   d->ItemToPublish = PublishItem;
   d->PublishIfNeeded();
}
