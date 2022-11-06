#include "ISink.h"

QString ItemToJson(const TPublishItem &Item);


class THttpSink: public ISink {
public:
   struct TCfg {
      bool DryRun;
   };


   explicit THttpSink(TCfg Cfg);
   // Credentials & auth
   // Http Endpoint
   void Publish(const TPublishItem &item) const override;

private:
   const TCfg Cfg;
};
