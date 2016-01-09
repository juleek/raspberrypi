#ifndef TTEMP_POLLER_H
#define TTEMP_POLLER_H

#include <QString>
#include <QObject>
#include <QTime>

struct TSensorInfo {
   QString Path;
   QString Name;
   double MinPossibleTemp;
};

class TTempPoller: public QObject {
public:
   TTempPoller(TSensorInfo SensorInfo) noexcept;

signals:
   void NewTemperatureGot(QString ErrMsg, double Temp);

public slots:
   void Bootstrap();

private:
   void ScheduleNextMeasurement() noexcept;
   void ItsTimeToGetTemperature() noexcept;

   TSensorInfo SensorInfo;
   QTime LastGet;
   QTime Periodicity;

   Q_OBJECT
};

#endif
