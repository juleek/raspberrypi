#include "TSmsSender.h"

#include <QDebug>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QTextStream>

struct TCategory {
   ~TCategory() noexcept {
      if (NetworkReply)
         NetworkReply->deleteLater();
   }
   TCategoryInfo  Info;
   std::uint32_t  CategoryId;
   QDateTime      LastSentFinished;
   QNetworkReply *NetworkReply = nullptr;
};

class TSmsSenderPrivate {
public:
   QString                                      UrlTemplate;
   std::unordered_map<std::uint32_t, TCategory> Settings;
   QNetworkAccessManager                        NetworkAccessManager;

   TCategory *CanSendNow(std::uint32_t CategoryId, QTextStream &ErrStr) noexcept;
};
TCategory *TSmsSenderPrivate::CanSendNow(std::uint32_t CategoryId, QTextStream &ErrStr) noexcept {
   const auto It = Settings.find(CategoryId);
   if (It == Settings.end()) {
      ErrStr << "Not such category id: " << CategoryId;
      return nullptr;
   }

   const TCategory &Category = It->second;
   if (Category.NetworkReply) {
      ErrStr << "Already sending SMS within this category: " << CategoryId;
      return nullptr;
   }

   const QDateTime &Current          = QDateTime::currentDateTime();
   const QDateTime &NextMostSoonSend = Category.LastSentFinished.addMSecs(Category.Info.Period.msecsSinceStartOfDay());
   //   qDebug() << "Current:" << Current
   //            << ", NextMostSoonSend:" << NextMostSoonSend
   //            << ", Period:" << Category.Info.Period
   //            << ", Category.LastSentFinished:" << Category.LastSentFinished;
   if (NextMostSoonSend.msecsTo(Current) < 0) {
      ErrStr << "Attempt to send SMS too often for category: " << CategoryId
             << ", LastSentFinished: " << Category.LastSentFinished.toString() << ", Current: " << Current.toString()
             << ", Period: " << Category.Info.Period.toString();
      return nullptr;
   }

   return const_cast<TCategory *>(&Category);
}
namespace {
   void HandleFinishOfRequest(TCategory *                 Category,
                              QNetworkReply *             NetworkReply,
                              QNetworkReply::NetworkError NetworkError) noexcept {
      if (Category->NetworkReply == nullptr)
         return;
      if (Category->NetworkReply != NetworkReply) {
         qDebug() << "Mismatching NetworkReplys! Category->NetworkReply:" << Category->NetworkReply
                  << ", NetworkReply:" << NetworkReply;
         std::abort();
      }
      if (NetworkError != QNetworkReply::NoError) {
         qDebug() << "SMSSender: Error occurred:" << NetworkError
                  << "while executing request:" << Category->NetworkReply->request().url()
                  << "in CategoryId:" << Category->CategoryId;
      } else {
         const QString &Answer = Category->NetworkReply->readAll();
         if (Answer.startsWith("ERROR"))
            qDebug() << "Got error from SMS relay:" << Answer;
      }
      Category->NetworkReply->deleteLater();
      Category->NetworkReply     = nullptr;
      Category->LastSentFinished = QDateTime::currentDateTime();
   }
} // namespace





TSmsSender::TSmsSender(QString                                          Login,
                       QString                                          Password,
                       QString                                          SenderId,
                       std::unordered_map<std::uint32_t, TCategoryInfo> Setup) noexcept
    : d_ptr(new TSmsSenderPrivate) {
   Q_D(TSmsSender);
   d->UrlTemplate = "http://smsc.ru/sys/send.php?"
                    "login=" +
                    std::move(Login) + "&psw=" + std::move(Password) + "&sender=" + std::move(SenderId) + "&phones=%1" +
                    "&mes=%2";

   for (auto &KV : Setup) {
      std::uint32_t  CategoryId   = KV.first;
      TCategoryInfo &CategoryInfo = KV.second;
      TCategory &    Category     = d->Settings[CategoryId];
      Category.Info               = CategoryInfo;
      Category.CategoryId         = CategoryId;
   }
}


namespace {
   QString ReceiversToString(const QSet<QString> &Receivers) noexcept {
      QString Result;
      for (const QString &Receiver : Receivers)
         Result += Receiver + ";";
      return Result;
   }
} // namespace
void TSmsSender::Send(std::uint32_t CategoryId, const QString &Message, const QSet<QString> &Receivers) noexcept {
   Q_D(TSmsSender);
   QString     ErrStr;
   QTextStream ErrStream(&ErrStr);
   TCategory * Category = d->CanSendNow(CategoryId, ErrStream);
   if (ErrStr.isEmpty() == false) {
      qDebug() << "Can not send SMS:" << ErrStr << ". SMS text:" << Message;
      return;
   }
   if (Category == nullptr) {
      qDebug() << "Can not send SMS: Category == nullptr!"
               << ". SMS text:" << Message;
      std::abort();
   }

   const QString &StrReceivers = ReceiversToString(Receivers);
   QString        StrUrl       = d->UrlTemplate.arg(StrReceivers).arg(Message);
   QUrl           Url(StrUrl);
   if (Url.isValid() == false) {
      qDebug() << "Url:" << Url << "is not valid!"
               << ". SMS text:" << Message;
      std::abort();
   }

   qDebug() << "Sending SMS:" << Url;
   //   Category->LastSentFinished = QTime::currentTime();
   //   return;

   QNetworkRequest NetworkRequest;
   NetworkRequest.setUrl(Url);

   Category->NetworkReply = d->NetworkAccessManager.get(NetworkRequest);
   QObject::connect(Category->NetworkReply, &QNetworkReply::finished, [ d, Category, nr = Category->NetworkReply ]() {
      HandleFinishOfRequest(Category, nr, QNetworkReply::NoError);
   });
   QObject::connect(Category->NetworkReply,
                    static_cast<void (QNetworkReply::*)(QNetworkReply::NetworkError)>(&QNetworkReply::error),
                    [ d, Category, nr = Category->NetworkReply ](QNetworkReply::NetworkError NetworkError) {
                       HandleFinishOfRequest(Category, nr, NetworkError);
                    });
}
