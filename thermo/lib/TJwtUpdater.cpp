#include "TJwt.h"
#include "memory.h"

#include <QBuffer>
#include <QJsonDocument>
#include <QJsonObject>
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




TJwtUpdater::TJwtUpdater(TCfg c): Cfg(std::move(c)) {
   Nam = MakeUnique();
}

TJwtUpdater::~TJwtUpdater() = default;


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


QString ParseIdTokenFromJson(const QByteArray &HttpBody) {
   QJsonParseError     JsonParseError;
   const QJsonDocument JsonDocument = QJsonDocument::fromJson(HttpBody, &JsonParseError);
   if(JsonParseError.error != QJsonParseError::NoError) {
      qDebug() << "TJwtUpdater: Failed to parse response as JSON:" << JsonParseError.errorString();
      return {};
   }
   if(JsonDocument.isObject() == false) {
      qDebug() << "TJwtUpdater: The root of JSON document is not an object";
      return {};
   }

   const QJsonValue &IdTokenValue = JsonDocument.object()["id_token"];
   if(IdTokenValue.isUndefined()) {
      qDebug() << "TJwtUpdater: The root of JSON document does not have \"id_token\"";
      return {};
   }
   if(IdTokenValue.isString() == false) {
      qDebug() << "TJwtUpdater: The root of JSON document has \"id_token\", but it is not a string";
      return {};
   }
   return IdTokenValue.toString();
}

namespace {

   void HandleResponse(QNetworkReply *Reply, TJwtUpdater *Self) {
      const QByteArray Data = Reply->readAll();
      qDebug() << "TJwtUpdater: Got reply for url:" << Reply->url() << "Headers:" << Reply->rawHeaderPairs()
               << "status:" << Reply->error() << "content:" << Data.first(std::min(Data.size(), 1024ll));

      if(Reply->error() != QNetworkReply::NoError) {
         qDebug() << "TJwtUpdater: Got error:" << Reply->error() << ", error:" << Reply->errorString();
         return;
      }

      const QString &Token = ParseIdTokenFromJson(Data);
      if(Token.isEmpty())
         return;

      qDebug() << "TJwtUpdater: Obtained new token:" << Token;

      emit Self->NewTokenObtained(Token);
   }
}   // namespace
void TJwtUpdater::OnResponse(QNetworkReply *Reply) {
   // emit NewTemperatureGot(std::get<0>(ErrStrAndTemp), std::get<1>(ErrStrAndTemp));
   HandleResponse(Reply, this);
   Reply->deleteLater();
   ScheduleNextMeasurement();
}

void TJwtUpdater::OnSslError(QNetworkReply *Reply, const QList<QSslError> &Errors) {
   qDebug() << "TJwtUpdater: Got SSL Error for url:" << Reply->url() << ":" << Errors;
}


void TJwtUpdater::OnTimerShot() {
   LastGet = QDateTime::currentDateTime();

   TJwt Jwt = {.Algo           = TJwt::RS256,
               .Audience       = "https://www.googleapis.com/oauth2/v4/token",
               .TargetAudience = Cfg.FunctionName,
               .Sub            = Cfg.AccountEmail,
               .Iss            = Cfg.AccountEmail};

   // Reason for this copy is that QBuffer wants non-const QByteArray in its ctor
   // but Cfg.PrivateKey is (deliberately) const.
   QByteArray NonConstPrivateKey = Cfg.PrivateKey;
   QBuffer    KeyStream          = {&NonConstPrivateKey};
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

   QNetworkReply *Reply = Nam->post(Request, Body.toUtf8());
   connect(Reply, &QNetworkReply::finished, [this, Reply]() { OnResponse(Reply); });
   connect(Reply, &QNetworkReply::sslErrors, [this, Reply](const QList<QSslError> &Errors) { OnSslError(Reply, Errors); });
}
