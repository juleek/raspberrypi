#include "TTempPoller.h"

#include <QThread>
#include <QDebug>
#include <QTimer>

TTempPoller::TTempPoller(TSensorInfo si) noexcept {
   SensorInfo = std::move(si);
   Periodicity = QTime(0, 0, 1);
}

void TTempPoller::Bootstrap() {
   qDebug().nospace() << "TempPoller started (and working) in thread: " << QThread::currentThreadId()
                      << ", Path:" << SensorInfo.Path << ", Name:" << SensorInfo.Name;
   ScheduleNextMeasurement();
}

void TTempPoller::ScheduleNextMeasurement() noexcept {
   const QTime &Current = QTime::currentTime();
   const QTime &NextGet = LastGet.addSecs(Periodicity.second());
   int MSecs = Current.msecsTo(NextGet);
   if(MSecs <= 0)
      MSecs = 0;
   QTimer::singleShot(MSecs, [this]() {
      ItsTimeToGetTemperature();
      LastGet = QTime::currentTime();
      ScheduleNextMeasurement();
   });
}

void TTempPoller::ItsTimeToGetTemperature() noexcept {
   emit NewTemperatureGot(SensorInfo.Name, "err", 12);
}

