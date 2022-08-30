#pragma once

#include <QHash>
#include <QString>

struct TPublishItem {
   QHash<QString, double> NameToTemp;
   QString                ErrorString;
};
