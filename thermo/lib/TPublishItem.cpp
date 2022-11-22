#include "TPublishItem.h"

#include <QDebug>

const QLatin1String TPublishItem::NAME_TO_TEMP_KEY = QLatin1String("NameToTemp");
const QLatin1String TPublishItem::ERROR_MSG_KEY    = QLatin1String("ErrorString");

QDebug operator<<(QDebug Out, const TPublishItem &PublishItem) {
   Out.nospace() << "NameToTemp: " << PublishItem.NameToTemp;
   if(PublishItem.ErrorString.isEmpty() == false)
      Out.nospace() << "ErrorString: " << PublishItem.ErrorString;
   return Out;
}
