#include "TTempPoller.h"

#include <QDebug>
#include <QFile>
#include <QString>
#include <QThread>
#include <QTimer>
#include <tuple>

namespace {
   std::tuple<QString, double> ParseTempFromLine(QString LineWithTemp) {
      if (LineWithTemp.endsWith("\n"))
         LineWithTemp.chop(1);

      QString BeforeTemp = " t=";
      int     Pos        = LineWithTemp.indexOf(BeforeTemp);
      Pos                = Pos + BeforeTemp.size();
      QString LineTemp   = LineWithTemp.right(LineWithTemp.size() - Pos);

      bool   Ok;
      double Temperature = LineTemp.toDouble(&Ok) / 1000.;
      // qDebug() << LineTemp << Temperature;
      if (Ok == false)
         return {"Failed to convert " + LineTemp + " to double", 0};
      return {{}, Temperature};
   }
} // namespace

std::tuple<QString, double> ProcessAndParseTemp(const QString &FileName) {
   QFile File(FileName);
   bool  Ok;
   Ok = File.open(QIODevice::ReadOnly);
   if (Ok == false)
      return {"Could not open file " + FileName, 0};

   /// We can't readLine by line file, see https://bugreports.qt.io/browse/QTBUG-24367
   /// bool QIODevice::canReadLine() const [virtual]
   /// Returns true if a complete line of data can be read from the device; otherwise returns false.
   /// Note that unbuffered devices, which have no way of determining what can be read, always return false.
   ///
   /// so, just readAll:
   QString     Content = File.readAll();
   QTextStream StreamContent(&Content, QIODevice::ReadOnly);

   int     NumberOfLines = 0;
   QString Line;
   for (; !StreamContent.atEnd(); ++NumberOfLines)
      Line = StreamContent.readLine();

   if (NumberOfLines != 2)
      return {"NumberOfLines != 2", 0};

   const std::tuple<QString, double> Result = ParseTempFromLine(Line);
   return Result;
}



TTempPoller::TTempPoller(TSensorInfo si) noexcept {
   SensorInfo  = std::move(si);
   Periodicity = QTime(0, 0, 15);
}

void TTempPoller::Bootstrap() {
   qDebug().nospace() << "TempPoller started (and working) in thread: " << QThread::currentThreadId()
                      << ", Path: " << SensorInfo.Path << ", Name: " << SensorInfo.Name;
   ScheduleNextMeasurement();
}

void TTempPoller::ScheduleNextMeasurement() noexcept {
   const QTime &Current = QTime::currentTime();
   const QTime &NextGet = LastGet.addMSecs(Periodicity.msecsSinceStartOfDay());
   const int    MSecs   = std::min(0, Current.msecsTo(NextGet));
   QTimer::singleShot(MSecs, this, SLOT(OnTimerShot()));
}

void TTempPoller::OnTimerShot() {
   std::tuple<QString, double> ErrStrAndTemp = ProcessAndParseTemp(SensorInfo.Path);
   emit                        NewTemperatureGot(std::get<0>(ErrStrAndTemp), std::get<1>(ErrStrAndTemp));
   LastGet = QTime::currentTime();
   ScheduleNextMeasurement();
}
