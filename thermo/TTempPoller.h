#pragma once

#include <QObject>
#include <QString>
#include <QTime>

struct TSensorInfo {
   QString Path;
   QString Name;
   double  MinPossibleTemp;
};

std::tuple<QString, double> ProcessAndParseTemp(const QString &FileName);

class TTempPoller : public QObject {
public:
   TTempPoller(TSensorInfo SensorInfo) noexcept;

signals:
   void NewTemperatureGot(QString ErrMsg, double Temp);

public slots:
   void Bootstrap();

private slots:
   void OnTimerShot();

private:
   void ScheduleNextMeasurement() noexcept;

   TSensorInfo SensorInfo;
   QTime       LastGet;
   QTime       Periodicity;

   Q_OBJECT
};
