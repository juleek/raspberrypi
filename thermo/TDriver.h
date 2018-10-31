#pragma once

#include <QObject>
#include <memory>

struct TSensorInfo;
class TDriverPrivate;
class TDriver : public QObject {
public:
   TDriver(const std::vector<TSensorInfo> &SensorInfos) noexcept;
   ~TDriver();

signals:
   void BootstrapTempPollers();

private:
   Q_OBJECT
   std::unique_ptr<TDriverPrivate> d;
};
