#include "THttpSink.h"

#include "TJwtUpdater.h"

#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>

template <class T>
class AsKeyValueRange {
public:
   AsKeyValueRange(T &data): m_data {data} {}
   auto begin() {
      return m_data.keyValueBegin();
   }
   auto end() {
      return m_data.keyValueEnd();
   }

private:
   T &m_data;
};

QString ItemToJson(const TPublishItem &Item) {
   QJsonObject NameToTemp;
   for(const auto &[Name, Temp]: AsKeyValueRange(Item.NameToTemp)) {
      NameToTemp[Name] = Temp;
   }
   QJsonObject Root;
   Root[TPublishItem::NAME_TO_TEMP_KEY] = NameToTemp;
   Root[TPublishItem::ERROR_MSG_KEY]    = Item.ErrorString;
   Root[TPublishItem::TIME_KEY]         = QDateTime::currentDateTime().toString(Qt::ISODateWithMs);
   return QJsonDocument(Root).toJson();
}


// ===========================================================================================================


THttpSink::THttpSink(TCfg c, TJwtUpdater &JwtUp, QNetworkAccessManager &NetworkAccessManager):
    Cfg(std::move(c)), Nam(NetworkAccessManager) {
   connect(&JwtUp, &TJwtUpdater::NewTokenObtained, this, &THttpSink::OnNewJwtToken);
   ReqTimeoutTimer.setSingleShot(true);
}


void THttpSink::OnNewJwtToken(const QString &Token) {
   qDebug() << "THttpSink::OnNewJwtToken: Got new token:" << Token;
   JwtToken = Token;
}



void THttpSink::OnResponse(QNetworkReply *Reply, const bool TimedOut) {
   ReqTimeoutTimer.stop();
   const int StatusCode = Reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
   qDebug().nospace() << "TJwtUpdater::OnResponse: Got reply for url: " << Reply->url() << ", TimedOut: " << TimedOut
                      << ", Headers: " << Reply->rawHeaderPairs() << ", status: " << StatusCode << " " << Reply->error()
                      << ", content: " << Reply->read(1024);

   Reply->deleteLater();
}
void THttpSink::OnSslError(QNetworkReply *Reply, const QList<QSslError> &Errors) const {
   qDebug() << "THttpSink::OnSslError: Failed to establish SSL connection: Got SSL Error for url:" << Reply->url() << ":"
            << Errors;
}


void THttpSink::Publish(const TPublishItem &Item) {
   qDebug() << "THttpSink::Publish: thread:" << (void *)thread();

   const QTime TIMEOUT = QTime(0, 1, 0);

   if(JwtToken.isEmpty()) {
      qDebug() << "THttpSink::Publish: Failed to publish item:" << Item << ": jwt token is empty";
      return;
   }

   QNetworkRequest Request(QUrl(Cfg.FuncHttpEndPoint));
   // Request.setTransferTimeout(TIMEOUT.msecsSinceStartOfDay());
   Request.setRawHeader("Authorization", ("Bearer " + JwtToken).toUtf8());
   Request.setRawHeader("Content-Type", "application/json");

   const QString Body = ItemToJson(Item);

   qDebug().nospace() << "THttpSink::Publish: " << (Cfg.DryRun ? "NOT " : "") << "Sending data to: " << Request.url()
                      << " with headers: " << Request.rawHeaderList() << " and body: " << Body;

   if(Cfg.DryRun)
      return;

   QNetworkReply *Reply = Nam.post(Request, Body.toUtf8());
   ReqTimeoutTimer.start(TIMEOUT.msecsSinceStartOfDay());
   connect(&ReqTimeoutTimer, &QTimer::timeout, [this, Reply]() { OnResponse(Reply, true); });
   connect(Reply, &QNetworkReply::finished, [this, Reply]() { OnResponse(Reply, false); });
   connect(Reply, &QNetworkReply::sslErrors, [this, Reply](const QList<QSslError> &Errors) { OnSslError(Reply, Errors); });
}
