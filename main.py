import os
import sys
import time
import datetime
import urllib.request

os.system('modprobe w1-gpio')
os.system('modprobe w1-therm')

# Constants:
TempPath = '/sys/bus/w1/devices/28-000005eac50a/w1_slave'
MinPossibleTemperature = 15
RegularSendingHour = 22 # hours of every day

MinDurationBetweenSMSSends = 20 # minutes
SMSPassword = ""
RegularReceivers = ["+79647088442", "+79037081325"]
AdditionalReceivers = ["+79647088442", "+79037081325"]


LastSendSMSTime = datetime.datetime(2000, 1, 1)
def SendSMS(MessageToSend, Receivers = None):
    global LastSendSMSTime
    Now = datetime.datetime.now()
    if Now - LastSendSMSTime < datetime.timedelta(0, 0, 0, 0, MinDurationBetweenSMSSends):
        print("You are attempting to send SMS to often, ignoring request...")
        return;

    if Receivers == None:
        Receivers = RegularReceivers

    Login = "dimanne"

    Url = "http://smsc.ru/sys/send.php?"
    Url += "login=" + Login
    Url += "&psw=" + SMSPassword
    Url += "&sender=" + "Tarasovka"
    Url += "&phones="
    for PhoneNumber in RegularReceivers:
        Url += PhoneNumber + ";"
    Url += "&mes=" + MessageToSend
    #print(Url)

    SendResult = urllib.request.urlopen(Url)
    LastSendSMSTime = datetime.datetime.now()
    #print(LastSendSMSTime)


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



def Usage():
    print("Usage: main.py <sms_path>")




FirstTime = True
MinT = 0
MaxT = 0

def UpdateStats(Temperature):
    global FirstTime
    global MinT
    global MaxT

    if FirstTime == True:
        MinT = Temperature
        MaxT = Temperature
        FirstTime = False
        return

    if MinT > Temperature:
        MinT = Temperature

    if MaxT < Temperature:
        MaxT = Temperature




AlreadySent = False
def SendSMSWhithStats(CurrentTemperature):
    global AlreadySent
    global RegularSendingHour
    Now = datetime.datetime.now()
    if Now.hour == RegularSendingHour and AlreadySent == False:
        SendSMS("T = " + str(CurrentTemperature) + ", Min = " + str(MinT) + ", Max = " + str(MaxT))
        FirstTime = True
        AlreadySent = True

    if Now.hour == RegularSendingHour + 1:
        AlreadySent = False


# ======================================================== Main() ========================================================

# Parse command-line arguments:
#print("Number of arguments: " + str(len(sys.argv)) + " arguments.")
#print("Argument List: " + str(sys.argv))
if len(sys.argv) != 2:
    Usage()
    exit
SMSPassword = sys.argv[1]

#SendSMS("Zarazina, chto delaesh'?")

while True:
    time.sleep(1)
    ParseResult = ParseTemp()
    print(ParseResult)
    if ParseResult[1] == False:
        SendSMS("ERROR AAAAA!!!!!!!!!!!!")
        continue

    if ParseResult[0] < MinPossibleTemperature:
        SendSMS("Current Temperature: " + str(ParseResult[0]) + ". MinPossibleTemperature: " + str(MinPossibleTemperature))

    UpdateStats(ParseResult[0])
    SendSMSWhithStats(ParseResult[0])

