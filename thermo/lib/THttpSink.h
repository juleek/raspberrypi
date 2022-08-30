#include "ISink.h"

class THttpSink: public ISink {
public:
   void Publish(const TPublishItem &item) const override;
};
