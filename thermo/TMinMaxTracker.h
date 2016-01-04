#ifndef TMYSTRUCT_H
#define TMYSTRUCT_H

struct TMinMaxTracker {
    void Update(double Tempr) {
        //MinMaxTracker->Min = 34;
        if(IsFirstTemp == true) {
            IsFirstTemp = false;
            Min = Tempr;
            Max = Tempr;
            return;
        }

        if(Tempr < Min)
            Min = Tempr;
        if(Tempr > Max)
            Max = Tempr;
    }

    double GetMin() {
        return Min;
    }

    double GetMax() {
        return Max;
    }


private:
    double Min;
    double Max;
    bool IsFirstTemp = true;
};



#endif // TMYSTRUCT_H
