#pragma once

#include "TPublishItem.h"

class ISink {
public:
   virtual ~ISink();

   virtual void Publish(const TPublishItem &item) const = 0;
};
