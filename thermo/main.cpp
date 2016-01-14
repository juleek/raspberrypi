#include "TDriver.h"

#include "TSmsSender.h"

#include <QCoreApplication>
#include <QCommandLineParser>
#include <QtDebug>

//void InProcTests(QString SmsPass) {
//   TSmsSender SmsSender("dimanne", SmsPass, "Tarasovka", { {0, {QTime(0, 0, 10)}} });
//   SmsSender.Send(0, "text of message", { {"+79647088442"} });
   //   QThread::sleep(25);
   //   SmsSender.Send(0, "123", { {"+79647088442"} });
   //   QThread::sleep(25);
   //   SmsSender.Send(0, "455", { {"+79647088442"} });
   //   QThread::sleep(25);
   //   SmsSender.Send(0, "678", { {"+79647088442"} });

//}

int main(int argc, char **argv) {
   QCoreApplication app(argc, argv);
   QCommandLineParser Parser;

   QCommandLineOption SMSPassOpt = QCommandLineOption("SMSPass", "Password for SMS gate", "String");
   Parser.addOption(SMSPassOpt);
   Parser.process(app);

   QString SMSPass = Parser.value(SMSPassOpt);
   qDebug() << "SMSPass:" << SMSPass;

   //InProcTests(SMSPass);

   const std::vector<TSensorInfo> SensorInfos = {
      { "/sys/bus/w1/devices/28-000005eac50a/w1_slave", "BottomTube", 12 },
      { "/sys/bus/w1/devices/28-000005eaddc2/w1_slave", "Ambient"   , 6  }
   };
   const QTime SendSMSStartTime = QTime(18, 15, 0);
   const QTime SendSMSEndTime   = QTime(19, 30, 0);

   new TDriver(SMSPass, SensorInfos, SendSMSStartTime, SendSMSEndTime);
   return app.exec();
}

