#include "THttpSink.h"

#include <QJsonDocument>
#include <QJsonObject>

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
   Root[TPublishItem::ERROR_MSG_KEY] = Item.ErrorString;
   return QJsonDocument(Root).toJson();
}

void THttpSink::Publish(const TPublishItem &item) const {
   Q_UNUSED(item);
}
