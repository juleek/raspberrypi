#include "TJwt.h"
#include "memory.h"

#include <QBuffer>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QTimer>
#include <QUrlQuery>
#include <TJwtUpdater.h>


QString FormUrlEncode(std::initializer_list<std::pair<QString, QString>> KVs) {
   QUrlQuery Helper;
   for(auto [Key, Value]: KVs) {
      Key.replace(' ', '+');
      Value.replace(' ', '+');
      Helper.addQueryItem(QUrl::toPercentEncoding(Key, "+"), QUrl::toPercentEncoding(Value, "+"));
   }
   return Helper.query(QUrl::FullyEncoded);
}




TJwtUpdater::TJwtUpdater(const QString &FunctionName, const QString &AccountEmail, const QByteArray &PrivateKey):
    FunctionName(FunctionName), AccountEmail(AccountEmail), PrivateKey(PrivateKey) {
   Nam = MakeUnique();
   connect(Nam.get(), &QNetworkAccessManager::finished, this, &TJwtUpdater::OnResponse);
}

namespace {
   const QTime TOKEN_LIFETIME = QTime(1, 0, 0);
   const QTime TIMEOUT        = QTime(0, 1, 0);
}   // namespace

void TJwtUpdater::ScheduleNextMeasurement() {
   // PERIODICITY is one third of TOKEN_LIFETIME (looks like Qt does not have a good API)
   static const QTime PERIODICITY = QTime::fromMSecsSinceStartOfDay(
       std::max(TIMEOUT.msecsSinceStartOfDay(), TOKEN_LIFETIME.msecsSinceStartOfDay() / 3 - TIMEOUT.msecsSinceStartOfDay()));

   const QDateTime &Current = QDateTime::currentDateTime();
   const QDateTime &NextGet = LastGet.addMSecs(PERIODICITY.msecsSinceStartOfDay());
   const int        MSecs   = std::max(0ll, Current.msecsTo(NextGet));
   QTimer::singleShot(MSecs, [this]() { OnTimerShot(); });
}

void TJwtUpdater::Start() {
   qDebug() << "TJwtUpdater::Start: Starting JwtUpdater...";
   OnTimerShot();
}

void TJwtUpdater::OnResponse(QNetworkReply *Reply) {
   const QByteArray Data = Reply->readAll();
   qDebug() << "TJwtUpdater: Got reply for url:" << Reply->url() << "Headers:" << Reply->rawHeaderPairs()
            << "status:" << Reply->error() << "content:" << Data.first(std::min(Data.size(), 1024ll));
   // emit NewTemperatureGot(std::get<0>(ErrStrAndTemp), std::get<1>(ErrStrAndTemp));
   Reply->deleteLater();
   ScheduleNextMeasurement();
}

void TJwtUpdater::OnTimerShot() {
   LastGet = QDateTime::currentDateTime();

   TJwt Jwt = {.Algo           = TJwt::RS256,
               .Audience       = "https://www.googleapis.com/oauth2/v4/token",
               .TargetAudience = FunctionName,
               .Sub            = AccountEmail,
               .Iss            = AccountEmail};

   QBuffer KeyStream = {&PrivateKey};
   KeyStream.open(QBuffer::ReadOnly);
   const QString SignedToken = Jwt.ComposeSignedToken(KeyStream);

   QNetworkRequest Request(QUrl("https://www.googleapis.com/oauth2/v4/token"));
   Request.setTransferTimeout(TIMEOUT.msecsSinceStartOfDay());
   Request.setRawHeader("Authorization", ("Bearer " + SignedToken).toUtf8());
   Request.setRawHeader("Content-Type", "application/x-www-form-urlencoded");

   const QString Body =
       FormUrlEncode({{"grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"}, {"assertion", SignedToken}});

   qDebug() << "TJwtUpdater: Sending request to url:" << Request.url() << "with headers:" << Request.rawHeaderList()
            << "and body:" << Body;

   Nam->post(Request, Body.toUtf8());
}
