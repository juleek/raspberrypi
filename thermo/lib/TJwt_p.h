#pragma once

#include "TJwt.h"

#include <QDateTime>

class TJwtPrivate {
public:
   TJwt::TAlgo Algo;
   QDateTime   IssuedAt;
   QDateTime   Expiration;
   QString     Audience;
   QString     TargetAudience;
   QString     Sub;
   QString     Iss;
};
