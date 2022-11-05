#pragma once

#include <QDateTime>
#include <QObject>


class QNetworkAccessManager;
class QNetworkReply;
class QSslError;


QString FormUrlEncode(std::initializer_list<std::pair<QString, QString>> KVs);
QString ParseIdTokenFromJson(const QByteArray &HttpBody);

class TJwtUpdater: public QObject {
public:
   TJwtUpdater(const QString &FunctionName, const QString &AccountEmail, const QByteArray &PrivateKey);
   ~TJwtUpdater();


signals:
   void NewTokenObtained(const QString &Token);

public slots:
   void Start();

private:
   void OnResponse(QNetworkReply *reply);
   void OnSslError(QNetworkReply *reply, const QList<QSslError> &Errors);

   void      OnTimerShot();
   void      ScheduleNextMeasurement();
   QDateTime LastGet;

   std::unique_ptr<QNetworkAccessManager> Nam;


   const QString FunctionName;
   const QString AccountEmail;
   QByteArray    PrivateKey;

   Q_OBJECT
};
