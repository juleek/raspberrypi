#ifndef TSMS_SENDER_H
#define TSMS_SENDER_H

#include <QObject>
#include <QString>
#include <QTime>
#include <QSet>

#include <unordered_map>
#include <unordered_set>

struct TCategoryInfo {
   QTime Period;
};

class TSmsSenderPrivate;
class TSmsSender: public QObject {
public:
   TSmsSender(QString Login, QString Password, QString SenderId,
              std::unordered_map<std::uint32_t, TCategoryInfo> Setup) noexcept;

public slots:
   void Send(std::uint32_t CategoryId, const QString &Message, const QSet<QString> &Receivers) noexcept;

private:
   Q_OBJECT
   Q_DECLARE_PRIVATE(TSmsSender)
   TSmsSenderPrivate  * const d_ptr;
};

#endif
