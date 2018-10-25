#pragma once
#include <QString>
#include <memory>

struct TGCMqttSetup {
   QString Host     = "mqtt.googleapis.com";
   int     Port     = 8883;
   QString UserName = {"asdf"};
   QString Location = "europe-west1";

   QString ProjectId;
   QString DeviceId;
   QString RegistryId;
   QString PrivateKeyPath;

   QString ClientId() const;
   QString Topic() const;
};

class TGCMqttPrivate;
class TGCMqtt {
public:
   TGCMqtt(const TGCMqttSetup &Setup);
   ~TGCMqtt();

   void Publish(double BottomTube, double Ambient, const QString &ErrorString = {});

private:
   std::unique_ptr<TGCMqttPrivate> d;
};
