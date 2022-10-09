#pragma once

#include <QObject>
#include <QDateTime>


class QNetworkAccessManager;
class QNetworkReply;


QString FormUrlEncode(std::initializer_list<std::pair<QString, QString>> KVs);

class TJwtUpdater: public QObject {
public:
   TJwtUpdater(const QString &FunctionName, const QString &AccountEmail, const QByteArray &PrivateKey);

signals:
   void NewTokenObtained(const QString &Token);

public slots:
   void Start();

private slots:
   void OnResponse(QNetworkReply *reply);

private:
   void      OnTimerShot();
   void      ScheduleNextMeasurement();
   QDateTime LastGet;

   std::unique_ptr<QNetworkAccessManager> Nam;


   const QString FunctionName;
   const QString AccountEmail;
   QByteArray    PrivateKey;
};
