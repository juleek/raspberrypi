#ifndef TDRIVER_H
#define TDRIVER_H

#include "TTempPoller.h"

#include <QString>
#include <QThread>
#include <QSocketNotifier>


#include <vector>
#include <memory>


struct TTempPollerAndThread {
   TTempPollerAndThread(TSensorInfo SensorInfo) noexcept;
   ~TTempPollerAndThread() noexcept;

   TTempPoller TempPoller;
private:
   QThread     Thread;
};
using TTempPollerAndThreadPtr = std::unique_ptr<TTempPollerAndThread>;


class QSocketNotifier;
class TDriver: public QObject {
public:
   TDriver(std::vector<TSensorInfo> SensorInfos) noexcept;

signals:
   void BootstrapTempPollers();

private:
   Q_OBJECT
   void OnNewTemperatureGot(QString SensorName, QString ErrStr, double Temp) noexcept;

   std::vector<TTempPollerAndThreadPtr> TempPollers;

   void InitSignalHandlers() noexcept;
   void OnSigInt();
   std::unique_ptr<QSocketNotifier> SigIntSocketNotifier;
};

#endif
