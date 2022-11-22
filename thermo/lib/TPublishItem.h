#pragma once

#include <QHash>
#include <QString>
#include <QStringView>

struct TPublishItem {
   static const QLatin1String NAME_TO_TEMP_KEY;
   static const QLatin1String ERROR_MSG_KEY;

   QHash<QString, double> NameToTemp;
   QString                ErrorString;
};

QDebug operator<<(QDebug Out, const TPublishItem &PublishItem);
