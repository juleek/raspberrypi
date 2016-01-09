#ifndef TDRIVER_H
#define TDRIVER_H

#include "TTempPoller.h"
#include "TSmsSender.h"
#include "TMinMaxTracker.h"

#include <QString>
#include <QThread>
#include <QSocketNotifier>


#include <vector>
#include <memory>


struct TTempPollerWrapper {
   TTempPollerWrapper(TSensorInfo SensorInfo) noexcept;
   ~TTempPollerWrapper() noexcept;

   TSensorInfo    SensorInfo;
   TMinMaxTracker MinMaxTracker;
   TTempPoller    TempPoller;
private:
   QThread        Thread;
};
using TTempPollerAndThreadPtr = std::unique_ptr<TTempPollerWrapper>;


class QSocketNotifier;
class TDriver: public QObject {
public:
   TDriver(QString SMSPass, std::vector<TSensorInfo> SensorInfos, QTime SendSMSStartTime, QTime SendSMSEndTime) noexcept;

signals:
   void BootstrapTempPollers();

private:
   Q_OBJECT
   void OnNewTemperatureGot(TTempPollerWrapper *Wrapper, QString ErrStr, double Temp) noexcept;

   std::unique_ptr<TSmsSender>          m_SmsSender;
   std::vector<TTempPollerAndThreadPtr> m_TempPollers;
   QTime m_SendSMSStartTime;
   QTime m_SendSMSEndTime;
   bool m_AllreadySent = false;

   void InitSignalHandlers() noexcept;
   void OnSigInt();
   std::unique_ptr<QSocketNotifier> m_SigIntSocketNotifier;
};

#endif
