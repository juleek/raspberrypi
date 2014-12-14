import os
import sys
import time
import datetime
import urllib.request
import traceback, signal, pprint

os.system('modprobe w1-gpio')
os.system('modprobe w1-therm')

# Constants:
TempPath1 = '/sys/bus/w1/devices/28-000005eac50a/w1_slave'
TempPath2 = '/sys/bus/w1/devices/28-000005eaddc2/w1_slave'
MinPossibleTemperature = 15
RegularSendingHour = 19 # hours of every day

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


def ReadFile(TempPath):
    File = open(TempPath, 'r')
    Lines = File.readlines()
    File.close()
    return Lines

def ParseTemp(TempPath):
    Lines = ReadFile(TempPath)
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


class TSensor:
    FirstTime = True
    MinT = 0
    MaxT = 0
    TimeOfMinT = ""
    TimeOfMaxT = ""
    TempPath = ""
    CurrentTemperature = 0

    def __init__(self, TempPath):
        self.TempPath = TempPath


    def UpdateStats(self, Temperature):
        Now = datetime.datetime.now().strftime("%H:%M:%S")
        if self.FirstTime == True:
            self.MinT = Temperature
            self.MaxT = Temperature
            self.TimeOfMinT = Now
            self.TimeOfMaxT = Now
            self.FirstTime = False
            return

        if self.MinT > Temperature:
            self.MinT = Temperature
            self.TimeOfMinT = Now

        if self.MaxT < Temperature:
            self.MaxT = Temperature
            self.TimeOfMaxT = Now

    def ParseAndUpdate(self):
        ParseResult = ParseTemp(self.TempPath)
        print(self.TempPath + " " + str(ParseResult))
        if ParseResult[1] == False:
            SendSMS("ERROR AAAAA!!!!!!!!!!!!")
            return

        self.CurrentTemperature = ParseResult[0]

        if ParseResult[0] < MinPossibleTemperature:
            SendSMS("Current Temperature: " + str(ParseResult[0]) + ". MinPossibleTemperature: " + str(MinPossibleTemperature))

        self.UpdateStats(ParseResult[0])





AlreadySent = False
def SendSMSWhithStats():
    global AlreadySent
    global RegularSendingHour
    Now = datetime.datetime.now()
    if (Now.hour == 15 or Now.hour == 17 or Now.hour == 19 or Now.hour == 21) and AlreadySent == False:
    #if Now.hour == RegularSendingHour and AlreadySent == False:
        SendSMS("Sensor1 T = "    + str(Sensor1.CurrentTemperature) +
                ", Min = "        + str(Sensor1.MinT) + "(" + str(Sensor1.TimeOfMinT) + ")" +
                ", Max = "        + str(Sensor1.MaxT) + "(" + str(Sensor1.TimeOfMaxT) + ")."
                " Sensor2 T = "   + str(Sensor2.CurrentTemperature) +
                ", Min = "        + str(Sensor2.MinT) + "(" + str(Sensor2.TimeOfMinT) + ")" +
                ", Max = "        + str(Sensor2.MaxT) + "(" + str(Sensor2.TimeOfMaxT) + ")"
                )
        FirstTime = True
        AlreadySent = True

    if Now.hour == 16 or Now.hour == 18 or Now.hour == 20:
    #if Now.hour == RegularSendingHour + 1:
        AlreadySent = False



def Debug():
    print("\n\nDebug Dump")
    pprint.pprint(globals())

    print("\n\nSensor1")
    pprint.pprint(vars(Sensor1))

    print("\n\nSensor2")
    pprint.pprint(vars(Sensor2))


def SignalHandler(sig, frame):
    Debug()


def ListenToSignal():
    signal.signal(signal.SIGUSR1, SignalHandler)  # Register handler



# ======================================================== Main() ========================================================

# Parse command-line arguments:
#print("Number of arguments: " + str(len(sys.argv)) + " arguments.")
#print("Argument List: " + str(sys.argv))
ListenToSignal()

if len(sys.argv) != 2:
    Usage()
    exit
SMSPassword = sys.argv[1]

Sensor1 = TSensor(TempPath1)
Sensor2 = TSensor(TempPath2)

while True:
    time.sleep(1)
    Sensor1.ParseAndUpdate()
    Sensor2.ParseAndUpdate()
    SendSMSWhithStats()

Debug()
