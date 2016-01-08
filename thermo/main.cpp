#include "TDriver.h"

#include "TSmsSender.h"

#include <QCoreApplication>

void InProcTests() {
   //   TSmsSender SmsSender("login", "pass", "tar", { {0, {QTime(0, 1, 0)}} });
   //   SmsSender.Send(0, "text of message", { {"+79647088442"} });
   //   QThread::sleep(25);
   //   SmsSender.Send(0, "123", { {"+79647088442"} });
   //   QThread::sleep(25);
   //   SmsSender.Send(0, "455", { {"+79647088442"} });
   //   QThread::sleep(25);
   //   SmsSender.Send(0, "678", { {"+79647088442"} });

}

int main(int argc, char **argv) {
   QCoreApplication app(argc, argv);

   InProcTests();

   std::vector<TSensorInfo> SensorInfos = {
      { "/home/dw", "Floor" }
   };

   new TDriver(SensorInfos);
   return app.exec();
}

