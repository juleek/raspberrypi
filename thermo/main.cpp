#include "TDriver.h"

#include "TSmsSender.h"

#include <QCoreApplication>
#include <QCommandLineParser>
#include <QtDebug>

void InProcTests(QString SmsPass) {
   TSmsSender SmsSender("dimanne", SmsPass, "Tarasovka", { {0, {QTime(0, 0, 10)}} });
   SmsSender.Send(0, "text of message", { {"+79647088442"} });
   //   QThread::sleep(25);
   //   SmsSender.Send(0, "123", { {"+79647088442"} });
   //   QThread::sleep(25);
   //   SmsSender.Send(0, "455", { {"+79647088442"} });
   //   QThread::sleep(25);
   //   SmsSender.Send(0, "678", { {"+79647088442"} });

}

int main(int argc, char **argv) {
   QCoreApplication app(argc, argv);
   QCommandLineParser Parser;

   QCommandLineOption SMSPassOpt = QCommandLineOption("SMSPass", "Password for SMS gate", "String");
   Parser.addOption(SMSPassOpt);
   Parser.process(app);

   QString SMSPass = Parser.value(SMSPassOpt);
   qDebug() << "SMSPass:" << SMSPass;

   InProcTests(SMSPass);

   std::vector<TSensorInfo> SensorInfos = {
      { "/home/dw", "Floor" }
   };

   new TDriver(SMSPass, SensorInfos);
   return app.exec();
}

