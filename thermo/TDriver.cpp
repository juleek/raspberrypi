#include "TDriver.h"
#include "TSmsCategoryIds.h"

#include <QDebug>
#include <QCoreApplication>

#include <iostream>

#include <sys/types.h>
#include <sys/socket.h>
#include <signal.h>
#include <unistd.h>


TTempPollerWrapper::TTempPollerWrapper(TSensorInfo si) noexcept: TempPoller(si) {
   SensorInfo = si;
   TempPoller.moveToThread(&Thread);
   Thread.start();
   qDebug() << "Created TempPoller and its thread (" << &Thread << ") from main thread" << QThread::currentThreadId();
}
TTempPollerWrapper::~TTempPollerWrapper() noexcept {
   qDebug() << "Stopping thread" << &Thread << "...";
   Thread.quit();
   Thread.wait();
   qDebug() << "Thread stopped";
}



namespace {
int SigIntFd[2];
void UnixSignalHandler(int) {
   char a = 1;
   ::write(SigIntFd[0], &a, sizeof(a));
   //std::cout << "UnixSignalHandler" << std::endl;
}
}

void TDriver::InitSignalHandlers() noexcept {
   int SocketPairCreated = socketpair(AF_UNIX, SOCK_STREAM, 0, SigIntFd);
   if(SocketPairCreated != 0)
      std::abort();

   m_SigIntSocketNotifier = std::make_unique<QSocketNotifier>(SigIntFd[1], QSocketNotifier::Read);
   connect(m_SigIntSocketNotifier.get(), &QSocketNotifier::activated, [this](int ) {
      OnSigInt();
   });

   struct sigaction Int;
   Int.sa_handler = UnixSignalHandler;
   sigemptyset(&Int.sa_mask);
   Int.sa_flags = 0;
   if(sigaction(SIGINT, &Int, 0) > 0)
      std::abort();
}
void TDriver::OnSigInt() {
   m_SigIntSocketNotifier->setEnabled(false);
   qDebug() << "Signal INT received - exiting...";
   delete this;
   qApp->quit();
}

namespace {
std::unordered_map<std::uint32_t, TCategoryInfo> SMSCategoriesDescription = {
   {TSmsCategoryIds::DailyStats  , { QTime(6, 0, 0)  }},
   {TSmsCategoryIds::ParsingError, { QTime(3, 0, 0)  }},
   {TSmsCategoryIds::Emergency   , { QTime(0, 55, 0) }}
};
QSet<QString> RegularReceivers = { "+79647088442", "+79037081325" };
void OutputSensorDailyStatsToStream(QTextStream &Stream, const TMinMaxTracker &MinMaxTracker) {

}
}

TDriver::TDriver(QString SMSPass,
                 std::vector<TSensorInfo> SensorInfos,
                 QTime SendSMSStartTime,
                 QTime SendSMSEndTime) noexcept {
   InitSignalHandlers();

   m_SendSMSStartTime = SendSMSStartTime;
   m_SendSMSEndTime = SendSMSEndTime;

   m_SmsSender = std::make_unique<TSmsSender>("dimanne", std::move(SMSPass), "Tarasovka", SMSCategoriesDescription);

   for(TSensorInfo &SensorInfo: SensorInfos) {
      TTempPollerAndThreadPtr Ptr = std::make_unique<TTempPollerWrapper>(std::move(SensorInfo));
      connect(&Ptr->TempPoller, &TTempPoller::NewTemperatureGot, [this, Wrapper = Ptr.get()](QString ErrStr, double Temp) {
         OnNewTemperatureGot(Wrapper, std::move(ErrStr), Temp);
      });
      connect(this, &TDriver::BootstrapTempPollers, &Ptr->TempPoller, &TTempPoller::Bootstrap);
      m_TempPollers.push_back(std::move(Ptr));
   }

   emit BootstrapTempPollers();
}

void TDriver::OnNewTemperatureGot(TTempPollerWrapper *Wrapper, QString ErrStr, double Temp) noexcept {
   qDebug() << "TDriver::OnNewTemperatureGot" << ErrStr << Temp;
   if(!ErrStr.isEmpty()) { // Error while parsing temperature
      QString SMSText = "Sensor " + Wrapper->SensorInfo.Path + ", " + Wrapper->SensorInfo.Name + " has ERROR: " + ErrStr;
      m_SmsSender->Send(TSmsCategoryIds::ParsingError, SMSText, RegularReceivers);
   } else {
      Wrapper->MinMaxTracker.Update(Temp);
      if(Temp < Wrapper->SensorInfo.MinPossibleTemp) {
         QString SMSText;
         m_SmsSender->Send(TSmsCategoryIds::Emergency, SMSText, RegularReceivers);
      }
   }

   // ---------------------- Daily stats reporting ----------------------
   const QTime &Current = QTime::currentTime();
   if(Current.msecsTo(m_SendSMSStartTime) < 0 && Current.msecsTo(m_SendSMSEndTime) > 0) {
      // Current time withing desirable time span
      if(m_AllreadySent == false) {
         QString SMSText;
         QTextStream Stream(&SMSText);
         for(const TTempPollerAndThreadPtr &W: m_TempPollers) {
            OutputSensorDailyStatsToStream(Stream, W->MinMaxTracker);
         }
         m_SmsSender->Send(TSmsCategoryIds::DailyStats, SMSText, RegularReceivers);
         m_AllreadySent = true;
      }
   } else {
      m_AllreadySent = false;
   }
}

