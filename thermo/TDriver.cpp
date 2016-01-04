#include "TDriver.h"

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

   SigIntSocketNotifier = std::make_unique<QSocketNotifier>(SigIntFd[1], QSocketNotifier::Read);
   connect(SigIntSocketNotifier.get(), &QSocketNotifier::activated, [this](int ) {
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
   SigIntSocketNotifier->setEnabled(false);
   qDebug() << "Signal INT received - exiting...";
   delete this;
   qApp->quit();
}

TDriver::TDriver(std::vector<TSensorInfo> SensorInfos) noexcept {
   InitSignalHandlers();

   for(TSensorInfo &SensorInfo: SensorInfos) {
      TTempPollerAndThreadPtr Ptr = std::make_unique<TTempPollerAndThread>(std::move(SensorInfo));
      connect(&Ptr->TempPoller, &TTempPoller::NewTemperatureGot, [this](QString SensorName, QString ErrStr, double Temp) {
         OnNewTemperatureGot(std::move(SensorName), std::move(ErrStr), Temp);
      });
      connect(this, &TDriver::BootstrapTempPollers, &Ptr->TempPoller, &TTempPoller::Bootstrap);
      TempPollers.push_back(std::move(Ptr));
   }

   emit BootstrapTempPollers();
}

void TDriver::OnNewTemperatureGot(QString SensorName, QString ErrStr, double Temp) noexcept {
   qDebug() << "TDriver::OnNewTemperatureGot" << SensorName << ErrStr << Temp;

}

