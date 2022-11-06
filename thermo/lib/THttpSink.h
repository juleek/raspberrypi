#include "ISink.h"

#include <QObject>

QString ItemToJson(const TPublishItem &Item);

class QNetworkAccessManager;
class QNetworkReply;
class QSslError;

class TJwtUpdater;

class THttpSink: public QObject, public ISink {
public:
   struct TCfg {
      const QString FuncHttpEndPoint;
      bool          DryRun;
   };

   explicit THttpSink(TCfg Cfg, TJwtUpdater &JwtUpdater, QNetworkAccessManager &NetworkAccessManager);

   void Publish(const TPublishItem &Item) const override;

public slots:
   void OnNewJwtToken(const QString &Token);


private:
   const TCfg             Cfg;
   QNetworkAccessManager &Nam;

   QString JwtToken;

   void OnResponse(QNetworkReply *Reply) const;
   void OnSslError(QNetworkReply *Reply, const QList<QSslError> &Errors) const;


   Q_OBJECT
};
