#pragma once

#include <QObject>
#include <memory>

struct TSensorInfo;
class ISink;

class TSensorsPoller: public QObject {
public:
   explicit TSensorsPoller(const std::vector<TSensorInfo> &SensorInfos, ISink &sink) noexcept;
   ~TSensorsPoller();

signals:
   void BootstrapTempPollers();

private slots:
   void OnNewTemperatureGot(const uint32_t Index, QString ErrStr, double Temp) noexcept;

private:
   Q_OBJECT
   struct TImpl;
   std::unique_ptr<TImpl> d;
};
