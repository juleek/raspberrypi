#include "TTempPoller.h"
#include "ParseTemp.h"

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
   const QTime &NextGet = LastGet.addMSecs(Periodicity.msecsSinceStartOfDay());
   int MSecs = Current.msecsTo(NextGet);
   if(MSecs <= 0)
      MSecs = 0;
   qDebug() << MSecs;
   QTimer::singleShot(MSecs, this, SLOT(OnTimerShot()));
   //QTimer::singleShot(MSecs, [this]() { OnTimerShot(); });
}

void TTempPoller::OnTimerShot() {
   ItsTimeToGetTemperature();
   LastGet = QTime::currentTime();
   ScheduleNextMeasurement();
}

void TTempPoller::ItsTimeToGetTemperature() noexcept {
   std::tuple<QString, double> ErrStrAndTemp = ProcessAndParseTemp(SensorInfo.Path);
   emit NewTemperatureGot(std::get<0>(ErrStrAndTemp), std::get<1>(ErrStrAndTemp));
}

