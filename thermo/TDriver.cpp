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
       : SensorInfo(si)
       , TempPoller(si) {

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

   const TSensorInfo SensorInfo;
   TTempPoller       TempPoller;



   void OnNewTemperatureGot(double t, const QString &err) {
      Temp = t;
      ErrStr += err;
      ++NumberOfConsecutiveReadings;
   }
   size_t GetNumberOfConsecutiveReadings() const {
      return NumberOfConsecutiveReadings;
   }
   std::pair<double, QString> GetTempAndErrStr() {
      const double  TempTemp      = Temp;
      const QString TempErrStr    = ErrStr;
      NumberOfConsecutiveReadings = 0;
      Temp                        = 0;
      ErrStr.clear();
      return {TempTemp, TempErrStr};
   }

private:
   QThread Thread;

   size_t  NumberOfConsecutiveReadings = 0;
   QString ErrStr;
   double  Temp;
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

namespace {
   void TempPollersToPublishItem(TPublishItem &PublishItem, std::vector<TTempPollerAndThreadPtr> &TempPollers) {
      for (TTempPollerAndThreadPtr &Sensor : TempPollers) {
         const size_t NumberOfConsecutiveReadings = Sensor->GetNumberOfConsecutiveReadings();
         double       Temp;
         QString      ErrStr;
         std::tie(Temp, ErrStr) = Sensor->GetTempAndErrStr();

         PublishItem.ErrorString += ErrStr;
         if (NumberOfConsecutiveReadings == 0)
            continue;

         PublishItem.NameToTemp[Sensor->SensorInfo.Name] = Temp;
      }
   }
} // namespace

void TDriverPrivate::OnNewTemperatureGot(TTempPollerWrapper *Wrapper, QString ErrStr, double Temp) noexcept {
   qDebug().nospace() << "TDriver::OnNewTemperatureGot:"
                      << " Name: " << Wrapper->SensorInfo.Name << ", T: " << Temp << ", Path: " << Wrapper->SensorInfo.Path
                      << ", ErrStr: " << ErrStr;
   Wrapper->OnNewTemperatureGot(Temp, ErrStr);

   const auto cmp = [](const auto &f, const auto &s) {
      return f->GetNumberOfConsecutiveReadings() < s->GetNumberOfConsecutiveReadings();
   };
   TTempPollerWrapper &MaxNumberSensor = **std::max_element(TempPollers.begin(), TempPollers.end(), cmp);
   TTempPollerWrapper &MinNumberSensor = **std::min_element(TempPollers.begin(), TempPollers.end(), cmp);



   static const size_t MAX_DIFFERENCE_BETWEEN_SENSORS = 4;
   if (MinNumberSensor.GetNumberOfConsecutiveReadings() == 0 &&
       MaxNumberSensor.GetNumberOfConsecutiveReadings() < MAX_DIFFERENCE_BETWEEN_SENSORS) {
      // We know that there is at least one lagging sensor (MinNumberSensor.GetNumberOfConsecutiveReadings() == 0)
      // but the diff between it and the most advanced one is less than thrashold => we are allowed to wait more time
      qDebug() << "Not publishing the reading, because there are other unread sensors: " << MinNumberSensor.SensorInfo.Name;
   } else {
      // Either all of the sensors have some data, or difference between them is larger than threshold
      TPublishItem PublishItem;
      if (MinNumberSensor.GetNumberOfConsecutiveReadings() == 0 &&
          MaxNumberSensor.GetNumberOfConsecutiveReadings() >= MAX_DIFFERENCE_BETWEEN_SENSORS) {
         PublishItem.ErrorString = QString("We were able to read %1 times from sensor %2:%3, but were "
                                           "unable to read once from sensor %4:%5")
                                       .arg(MaxNumberSensor.GetNumberOfConsecutiveReadings())
                                       .arg(MaxNumberSensor.SensorInfo.Name, MaxNumberSensor.SensorInfo.Path)
                                       .arg(MinNumberSensor.SensorInfo.Name, MinNumberSensor.SensorInfo.Path);
      }
      TempPollersToPublishItem(PublishItem, TempPollers);
      // TGCMqtt: Publishing: "{\"Ambient\":22.875,\"BottomTube\":22.875}"
      Mqtt->Publish(PublishItem);
   }
}

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
