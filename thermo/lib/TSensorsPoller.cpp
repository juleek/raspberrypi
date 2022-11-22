#include "TSensorsPoller.h"

#include "ISink.h"
#include "TPublishItem.h"
#include "TSensorPoller.h"
#include "memory.h"

#include <QDebug>
#include <QThread>
#include <iostream>


struct TPollerWithThread {
   TPollerWithThread(TSensorInfo si, const uint32_t Index) noexcept: SensorInfo(si), TempPoller(si, Index) {
      TempPoller.moveToThread(&Thread);
      Thread.start();
      qDebug().nospace() << "Created TempPoller and its thread (" << &Thread
                         << ") from main thread: " << QThread::currentThreadId();
   }
   ~TPollerWithThread() noexcept {
      qDebug() << "Stopping thread" << &Thread << "...";
      Thread.quit();
      Thread.wait();
      qDebug() << "Thread stopped";
   }

   const TSensorInfo SensorInfo;
   TSensorPoller     TempPoller;


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
using TPollerWithThreadPtr = std::unique_ptr<TPollerWithThread>;

void ConvertPollerDataToPublishItem(TPublishItem &PublishItem, TPollerWithThreadPtr &Poller) {
   const size_t NumberOfConsecutiveReadings = Poller->GetNumberOfConsecutiveReadings();
   double       Temp;
   QString      ErrStr;
   std::tie(Temp, ErrStr) = Poller->GetTempAndErrStr();

   PublishItem.ErrorString += ErrStr;
   if(NumberOfConsecutiveReadings == 0)
      return;

   PublishItem.NameToTemp[Poller->SensorInfo.Name] = Temp;
}



// ===========================================================================================================



struct TSensorsPoller::TImpl {
   std::vector<TPollerWithThreadPtr> TempPollers;
   ISink                            *Sink;
};



void TSensorsPoller::OnNewTemperatureGot(const uint32_t Index, QString ErrStr, double Temp) noexcept {
   TPollerWithThread *Wrapper = d->TempPollers[Index].get();
   qDebug() << "OnNewTemperatureGot: thread:" << (void *)thread() << (void *)QThread::currentThread();
   qDebug().nospace() << "TDriver::OnNewTemperatureGot:"
                      << " Name: " << Wrapper->SensorInfo.Name << ", T: " << Temp << ", Path: " << Wrapper->SensorInfo.Path
                      << ", ErrStr: " << ErrStr;

   // if (Wrapper->SensorInfo.Name == "Ambient")
   Wrapper->OnNewTemperatureGot(Temp, ErrStr);

   const auto cmp = [](const auto &f, const auto &s) {
      return f->GetNumberOfConsecutiveReadings() < s->GetNumberOfConsecutiveReadings();
   };
   TPollerWithThread &MaxNumberSensor = **std::max_element(d->TempPollers.begin(), d->TempPollers.end(), cmp);
   TPollerWithThread &MinNumberSensor = **std::min_element(d->TempPollers.begin(), d->TempPollers.end(), cmp);


   static const size_t MAX_DIFFERENCE_BETWEEN_SENSORS = 4;
   if(MinNumberSensor.GetNumberOfConsecutiveReadings() == 0 &&
      MaxNumberSensor.GetNumberOfConsecutiveReadings() < MAX_DIFFERENCE_BETWEEN_SENSORS) {
      // We know that there is at least one lagging sensor (MinNumberSensor.GetNumberOfConsecutiveReadings() == 0)
      // but the diff between it and the most advanced one is less than thrashold => we are allowed to wait more time
      qDebug() << "Not publishing the reading, because there are other unread sensors: " << MinNumberSensor.SensorInfo.Name;
      return;
   }

   // Either all of the sensors have some data, or difference between them is larger than threshold
   TPublishItem PublishItem;
   if(MinNumberSensor.GetNumberOfConsecutiveReadings() == 0 &&
      MaxNumberSensor.GetNumberOfConsecutiveReadings() >= MAX_DIFFERENCE_BETWEEN_SENSORS) {
      PublishItem.ErrorString = QString("We were able to read %1 times from sensor %2:%3, but were "
                                        "unable to read once from sensor %4:%5")
                                    .arg(MaxNumberSensor.GetNumberOfConsecutiveReadings())
                                    .arg(MaxNumberSensor.SensorInfo.Name, MaxNumberSensor.SensorInfo.Path)
                                    .arg(MinNumberSensor.SensorInfo.Name, MinNumberSensor.SensorInfo.Path);
   }

   for(TPollerWithThreadPtr &Poller: d->TempPollers)
      ConvertPollerDataToPublishItem(PublishItem, Poller);

   d->Sink->Publish(PublishItem);
}

TSensorsPoller::TSensorsPoller(const std::vector<TSensorInfo> &SensorInfos, ISink &Sink) noexcept: d(new TImpl) {
   d->Sink = &Sink;

   qDebug() << "TSensorsPoller: thread:" << (void *)thread();
   uint32_t Index = 0;
   for(const TSensorInfo &SensorInfo: SensorInfos) {
      TPollerWithThreadPtr Ptr = MakeUnique(SensorInfo, Index);
      ++Index;
      connect(&Ptr->TempPoller, &TSensorPoller::NewTemperatureGot, this, &TSensorsPoller::OnNewTemperatureGot);
      connect(this, &TSensorsPoller::BootstrapTempPollers, &Ptr->TempPoller, &TSensorPoller::Bootstrap);
      d->TempPollers.push_back(std::move(Ptr));
   }

   emit BootstrapTempPollers();
}
TSensorsPoller::~TSensorsPoller() = default;
