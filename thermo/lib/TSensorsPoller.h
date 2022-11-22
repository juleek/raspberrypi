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

private:
   Q_OBJECT
   struct TImpl;
   std::unique_ptr<TImpl> d;
};
