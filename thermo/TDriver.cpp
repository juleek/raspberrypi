#include "TDriver.h"
#include "MakeUnique.h"
#include "TGCMqtt.h"
#include "TTempPoller.h"

#include <QCoreApplication>
#include <QDebug>
#include <QSocketNotifier>
#include <QThread>
#include <iostream>
#include <signal.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>


struct TTempPollerWrapper {
   TTempPollerWrapper(TSensorInfo si) noexcept
       : TempPoller(si) {
      SensorInfo = si;
      TempPoller.moveToThread(&Thread);
      Thread.start();
      qDebug().nospace() << "Created TempPoller and its thread (" << &Thread
                         << ") from main thread: " << QThread::currentThreadId();
   }
   ~TTempPollerWrapper() noexcept {
      qDebug() << "Stopping thread" << &Thread << "...";
      Thread.quit();
      Thread.wait();
      qDebug() << "Thread stopped";
   }

   TSensorInfo SensorInfo;
   TTempPoller TempPoller;

private:
   QThread Thread;
};



using TTempPollerAndThreadPtr = std::unique_ptr<TTempPollerWrapper>;

class TDriverPrivate {
public:
   void OnNewTemperatureGot(TTempPollerWrapper *Wrapper, QString ErrStr, double Temp) noexcept;

   std::vector<TTempPollerAndThreadPtr> TempPollers;
   std::unique_ptr<TGCMqtt>             Mqtt;
   QTime                                SendSMSStartTime;
   QTime                                SendSMSEndTime;
   bool                                 AllreadySent = false;


   void InitSignalHandlers() noexcept;
   void OnSigInt();

   std::unique_ptr<QSocketNotifier> SigIntSocketNotifier;
};



namespace {
   int  SigIntFd[2];
   void UnixSignalHandler(int) {
      char a = 1;
      ::write(SigIntFd[0], &a, sizeof(a));
      // std::cout << "UnixSignalHandler" << std::endl;
   }
} // namespace

void TDriverPrivate::InitSignalHandlers() noexcept {
   int SocketPairCreated = socketpair(AF_UNIX, SOCK_STREAM, 0, SigIntFd);
   if (SocketPairCreated != 0)
      std::abort();

   SigIntSocketNotifier = MakeUnique(new QSocketNotifier(SigIntFd[1], QSocketNotifier::Read));
   QObject::connect(SigIntSocketNotifier.get(), &QSocketNotifier::activated, [this](int) { OnSigInt(); });

   struct sigaction Int;
   Int.sa_handler = UnixSignalHandler;
   sigemptyset(&Int.sa_mask);
   Int.sa_flags = 0;
   if (sigaction(SIGINT, &Int, 0) > 0)
      std::abort();
}
void TDriverPrivate::OnSigInt() {
   SigIntSocketNotifier->setEnabled(false);
   qDebug() << "Signal INT received - exiting...";
   // delete this;
   qApp->quit();
}

void TDriverPrivate::OnNewTemperatureGot(TTempPollerWrapper *Wrapper, QString ErrStr, double Temp) noexcept {
   qDebug().nospace() << "TDriver::OnNewTemperatureGot:"
                      << " Name: " << Wrapper->SensorInfo.Name << ", Path: " << Wrapper->SensorInfo.Path
                      << ", ErrStr: " << ErrStr << ", T: " << Temp;
   Mqtt->Publish(Temp, Temp, ErrStr);
   if (!ErrStr.isEmpty()) { // Error while parsing temperature
      // QString SMSText = "Sensor " + Wrapper->SensorInfo.Path + ", " + Wrapper->SensorInfo.Name + " has ERROR: " + ErrStr;
      // SmsSender->Send(TSmsCategoryIds::ParsingError, SMSText, RegularReceivers);
   } else {
      // Wrapper->MinMaxTracker.Update(Temp);
      // if (Temp < Wrapper->SensorInfo.MinPossibleTemp) {
      //    QString     SMSText;
      //    QTextStream Stream(&SMSText);
      //    OutputTooColdMessageToStream(Stream, Temp, Wrapper->SensorInfo.MinPossibleTemp, Wrapper->SensorInfo.Name);
      //    SmsSender->Send(TSmsCategoryIds::Emergency, SMSText, RegularReceivers);
      // }
   }

   // ---------------------- Daily stats reporting ----------------------
   // const QTime &Current = QTime::currentTime();
   // if (Current.msecsTo(SendSMSStartTime) < 0 && Current.msecsTo(SendSMSEndTime) > 0) {
   //    if (AllreadySent == false && AllSensorsHasMeasurements(TempPollers)) {
   //       QString     SMSText;
   //       QTextStream Stream(&SMSText);
   //       for (const TTempPollerAndThreadPtr &W : TempPollers) {
   //          OutputSensorDailyStatsToStream(SMSText, Stream, W->MinMaxTracker, W->SensorInfo.Name);
   //       }
   //       SmsSender->Send(TSmsCategoryIds::DailyStats, SMSText, RegularReceivers);
   //       AllreadySent = true;
   //    }
   // } else {
   //    AllreadySent = false;
   // }
}

namespace {
   // std::unordered_map<std::uint32_t, TCategoryInfo> SMSCategoriesDescription =
   //     {{TSmsCategoryIds::DailyStats, {QTime(6, 0, 0)}},
   //      {TSmsCategoryIds::ParsingError, {QTime(3, 0, 0)}},
   //      {TSmsCategoryIds::Emergency, {QTime(0, 55, 0)}}};
   // QSet<QString> RegularReceivers = {"+79647088442", "+79037081325"};
   // void          OutputSensorDailyStatsToStream(const QString &       Result,
   //                                              QTextStream &         Stream,
   //                                              const TMinMaxTracker &MinMaxTracker,
   //                                              const QString &       SensorName) {
   //    if (Result.isEmpty() == false)
   //       Stream << " ";
   //    Stream << SensorName << " T = " << MinMaxTracker.GetLast() << ", Min = " << MinMaxTracker.GetMin() << "("
   //           << MinMaxTracker.GetTimeOfMin().toString("hh:mm") << ")"
   //           << ", Max = " << MinMaxTracker.GetMax() << "(" << MinMaxTracker.GetTimeOfMax().toString("hh:mm") << ")"
   //           << ".";
   // }
   // void OutputTooColdMessageToStream(QTextStream &  Stream,
   //                                   const double   CurrentTemp,
   //                                   const double   MinPossibleTemp,
   //                                   const QString &SensorName) {
   //    Stream << SensorName << " T = " << CurrentTemp << ", lower than min possible: " << MinPossibleTemp << "!";
   // }
   // bool AllSensorsHasMeasurements(const std::vector<TTempPollerAndThreadPtr> &TempPollers) {
   //    for (const TTempPollerAndThreadPtr &W : TempPollers)
   //       if (W->MinMaxTracker.HasMeasurements() == false)
   //          return false;
   //    return true;
   // }
} // namespace

TDriver::TDriver(const std::vector<TSensorInfo> &SensorInfos, const TGCMqttSetup &MqttSetup) noexcept
    : d(new TDriverPrivate) {
   d->InitSignalHandlers();
   d->Mqtt = std::make_unique<TGCMqtt>(MqttSetup);

   for (const TSensorInfo &SensorInfo : SensorInfos) {
      TTempPollerAndThreadPtr Ptr = std::make_unique<TTempPollerWrapper>(SensorInfo);
      QObject::connect(&Ptr->TempPoller, &TTempPoller::NewTemperatureGot, [ this, W = Ptr.get() ](QString Err, double T) {
         d->OnNewTemperatureGot(W, std::move(Err), T);
      });
      QObject::connect(this, &TDriver::BootstrapTempPollers, &Ptr->TempPoller, &TTempPoller::Bootstrap);
      d->TempPollers.push_back(std::move(Ptr));
   }

   emit BootstrapTempPollers();
}
TDriver::~TDriver() = default;
