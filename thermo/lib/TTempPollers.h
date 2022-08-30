#pragma once

#include <QObject>
#include <memory>

struct TSensorInfo;
class ISink;

class TDriver: public QObject {
public:
   TDriver(const std::vector<TSensorInfo> &SensorInfos, const ISink &sink) noexcept;
   ~TDriver();

signals:
   void BootstrapTempPollers();

private:
   Q_OBJECT
   struct TImpl;
   std::unique_ptr<TImpl> d;
};
