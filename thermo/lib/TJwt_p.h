#pragma once

#include "TJwt.h"

#include <QDateTime>

class TJwtPrivate {
public:
   TJwt::TAlgo Algo;
   QDateTime   IssuedAt;
   QDateTime   Expiration;
   QString     Audience;
};
