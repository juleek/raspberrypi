#ifndef TMYSTRUCT_H
#define TMYSTRUCT_H

#include <QDateTime>

struct TMinMaxTracker {
   void Update(double Tempr) {
      //MinMaxTracker->Min = 34;
      Last = Tempr;

      if(IsFirstTemp == true) {
         IsFirstTemp = false;
         Min = Tempr;
         Max = Tempr;
         TimeOfMin = QDateTime::currentDateTime();
         TimeOfMax = QDateTime::currentDateTime();
         return;
      }

      if(Tempr < Min) {
         Min = Tempr;
         TimeOfMin = QDateTime::currentDateTime();
      }
      if(Tempr > Max) {
         Max = Tempr;
         TimeOfMax = QDateTime::currentDateTime();
      }
   }

   double GetLast() {
      return Last;
   }
   double GetMin() {
      return Min;
   }
   double GetMax() {
      return Max;
   }
   QDateTime GetTimeOfMin() {
      return TimeOfMin;
   }
   QDateTime GetTimeOfMax() {
      return TimeOfMax;
   }

private:
   QDateTime TimeOfMin;
   QDateTime TimeOfMax;
   double Min  = 0;
   double Max  = 0;
   double Last = 0;
   bool IsFirstTemp = true;
};



#endif // TMYSTRUCT_H
