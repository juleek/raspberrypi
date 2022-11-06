#pragma once

#include <QHash>
#include <QString>
#include <QStringView>

struct TPublishItem {
   static constexpr QStringView NAME_TO_TEMP_KEY = u"NameToTemp";
   static constexpr QStringView ERROR_MSG_KEY    = u"ErrorString";

   QHash<QString, double> NameToTemp;
   QString                ErrorString;
};

QDebug operator<<(QDebug Out, const TPublishItem &PublishItem);
