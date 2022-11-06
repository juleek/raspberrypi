#pragma once

#include <QObject>
#include <QString>
#include <QTime>

struct TSensorInfo {
   QString Path;
   QString Name;
};

class QIODevice;
std::tuple<QString, double> ParseTempFrom(QIODevice &input);

class TSensorPoller: public QObject {
public:
   TSensorPoller(TSensorInfo SensorInfo) noexcept;

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
