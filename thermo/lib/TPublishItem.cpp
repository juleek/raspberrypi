#include "TPublishItem.h"

#include <QDebug>

QDebug operator<<(QDebug Out, const TPublishItem &PublishItem) {
   Out.nospace() << "NameToTemp: " << PublishItem.NameToTemp;
   if(PublishItem.ErrorString.isEmpty() == false)
      Out.nospace() << "ErrorString: " << PublishItem.ErrorString;
   return Out;
}
