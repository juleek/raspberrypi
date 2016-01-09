#include "TDriver.h"
#include "TSmsCategoryIds.h"

#include <QDebug>
#include <QCoreApplication>

#include <iostream>

#include <sys/types.h>
#include <sys/socket.h>
#include <signal.h>
#include <unistd.h>


TTempPollerAndThread::TTempPollerAndThread(TSensorInfo SensorInfo) noexcept: TempPoller(std::move(SensorInfo)) {
   TempPoller.moveToThread(&Thread);
   Thread.start();
   qDebug() << "Created TempPoller and its thread (" << &Thread << ") from main thread" << QThread::currentThreadId();
}
TTempPollerAndThread::~TTempPollerAndThread() noexcept {
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
   {TSmsCategoryIds::DaylyStats, { QTime(6, 0, 0)  }},
   {TSmsCategoryIds::Emergency , { QTime(0, 55, 0) }}
};
}

TDriver::TDriver(QString SMSPass, std::vector<TSensorInfo> SensorInfos) noexcept {
   InitSignalHandlers();

   m_SmsSender = std::make_unique<TSmsSender>("dimanne", std::move(SMSPass), "Tarasovka", SMSCategoriesDescription);

   for(TSensorInfo &SensorInfo: SensorInfos) {
      TTempPollerAndThreadPtr Ptr = std::make_unique<TTempPollerAndThread>(std::move(SensorInfo));
      connect(&Ptr->TempPoller, &TTempPoller::NewTemperatureGot, [this](QString SensorName, QString ErrStr, double Temp) {
         OnNewTemperatureGot(std::move(SensorName), std::move(ErrStr), Temp);
      });
      connect(this, &TDriver::BootstrapTempPollers, &Ptr->TempPoller, &TTempPoller::Bootstrap);
      m_TempPollers.push_back(std::move(Ptr));
   }

   emit BootstrapTempPollers();
}

//TSmsSender *TDriver::SmsSender() const noexcept {
//   return m_SmsSender.get();
//}

void TDriver::OnNewTemperatureGot(QString SensorName, QString ErrStr, double Temp) noexcept {
   qDebug() << "TDriver::OnNewTemperatureGot" << SensorName << ErrStr << Temp;
}

