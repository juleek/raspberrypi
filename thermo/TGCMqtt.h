#pragma once
#include <QString>
#include <memory>
#include <QHash>

struct TGCMqttSetup {
   QString Host     = "mqtt.googleapis.com";
   int     Port     = 8883;
   QString UserName = {"asdf"};
   QString Location = "europe-west1";
   bool    DryRun   = false;

   QString ProjectId;
   QString DeviceId;
   QString RegistryId;
   QString PrivateKeyPath;


   QString ClientId() const;
   QString Topic() const;
};
QDebug operator<<(QDebug Out, const TGCMqttSetup &Setup);

struct TPublishItem {
  QHash<QString, double> NameToTemp;
  QString ErrorString;
};

class TGCMqttPrivate;
class TGCMqtt {
public:
   TGCMqtt(const TGCMqttSetup &Setup);
   ~TGCMqtt();

   void Publish(const TPublishItem &PublishItem);

private:
   std::unique_ptr<TGCMqttPrivate> d;
};
