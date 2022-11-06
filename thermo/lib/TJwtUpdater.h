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
   struct TCfg {
      const QString    FuncHttpEndPoint;
      const QString    AccountEmail;
      const QByteArray PrivateKey;
   };

   explicit TJwtUpdater(TCfg Cfg, QNetworkAccessManager &NetworkAccessManager);
   ~TJwtUpdater();


signals:
   void NewTokenObtained(const QString &Token);

public slots:
   void Start();

private:
   const TCfg             Cfg;
   QNetworkAccessManager &Nam;


   void OnResponse(QNetworkReply *reply);
   void OnSslError(QNetworkReply *reply, const QList<QSslError> &Errors);

   void      OnTimerShot();
   void      ScheduleNextMeasurement();
   QDateTime LastGet;

   Q_OBJECT
};
