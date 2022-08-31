#include "ISink.h"

QString ItemToJson(const TPublishItem &Item);

class THttpSink: public ISink {
public:
   // Credentials & auth
   // Http Endpoint
   void Publish(const TPublishItem &item) const override;
};
