import os
import time

os.system('modprobe w1-gpio')
os.system('modprobe w1-therm')

TempPath = '/sys/bus/w1/devices/28-000005eac50a/w1_slave'

def SendSMS(MessageToSend):
   print(MessageToSend)


def ReadFile():
    File = open(TempPath, 'r')
    Lines = File.readlines()
    File.close()
    return Lines

def ParseTemp():
    Lines = ReadFile()
    if Lines[0].strip()[-3:] != 'YES':
        return 0, False
    PosOfT = Lines[1].find('t=')
    if PosOfT == -1:
        return 0, False
    TemporString = Lines[1].strip()[PosOfT + 2:]
    Result = float(TemporString) / 1000.0
    return Result, True




while True:
    time.sleep(1)
    ParseResult = ParseTemp()
    if ParseResult[1] == False:
        SendSMS("ERROR AAAAA!!!!!!!!!!!!")
        continue

#print(ParseTemp())
