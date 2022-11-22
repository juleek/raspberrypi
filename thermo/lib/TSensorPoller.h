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
   TSensorPoller(TSensorInfo SensorInfo, const uint32_t Ind) noexcept;

signals:
   void NewTemperatureGot(const uint32_t Index, QString ErrMsg, double Temp);

public slots:
   void Bootstrap();

private slots:
   void OnTimerShot();

private:
   void ScheduleNextMeasurement() noexcept;

   TSensorInfo SensorInfo;
   uint32_t    Index;
   QTime       LastGet;
   QTime       Periodicity;

   Q_OBJECT
};
