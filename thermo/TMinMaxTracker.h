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

   double GetLast() const {
      return Last;
   }
   double GetMin() const {
      return Min;
   }
   double GetMax() const {
      return Max;
   }
   QDateTime GetTimeOfMin() const {
      return TimeOfMin;
   }
   QDateTime GetTimeOfMax() const {
      return TimeOfMax;
   }
   bool HasMeasurements() const {
      return IsFirstTemp == false;
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
