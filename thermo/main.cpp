#include "TDriver.h"

#include <QCoreApplication>

int main(int argc, char **argv) {
   QCoreApplication app(argc, argv);

   std::vector<TSensorInfo> SensorInfos = {
      { "/home/dw", "Floor" }
   };

   new TDriver(SensorInfos);
   return app.exec();
}

