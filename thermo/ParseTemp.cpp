#include "ParseTemp.h"
#include <QFile>
#include <QDebug>

std::tuple<QString, double> ParseTempFromLine(QString LineWithTemp) {
    if(LineWithTemp.endsWith("\n"))
        LineWithTemp.chop(1);

    QString BeforeTemp = " t=";
    int Pos = LineWithTemp.indexOf(BeforeTemp);
    Pos = Pos + BeforeTemp.size();
    QString LineTemp = LineWithTemp.right(LineWithTemp.size() - Pos);

    bool Ok;
    double Temperature = LineTemp.toDouble(&Ok) / 1000.;
    //qDebug() << LineTemp << Temperature;
    if(Ok == false) {
        return std::tuple<QString, double>("Conversion error occured", 0);
    }
    return std::tuple<QString, double>("", Temperature);
}


std::tuple<QString, double> ProcessAndParseTemp(QString FileName) {
    QFile File(FileName);
    bool Ok;
    Ok = File.open(QIODevice::ReadOnly);
    if(Ok == false) {
        File.close();
        return std::tuple<QString, double>("Could not open file", 0);
    }

    QString Content = File.readAll();
    File.close();
    QTextStream StreamContent(&Content, QIODevice::ReadOnly);

    int NumberOfLines = 0;
    QString Line;
    for(; !StreamContent.atEnd(); ++NumberOfLines) {
        //qDebug() << FileName << File.canReadLine() << Line;
        Line = StreamContent.readLine();
    }


    if(NumberOfLines != 2) {
        return std::tuple<QString, double>("NumberOfLines != 2", 0);
    }

    std::tuple<QString, double> Result = ParseTempFromLine(Line);
    return Result;
}
