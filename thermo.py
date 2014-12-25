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
RegularSendingHour = 19 # hours of every day

MinDurationBetweenSMSSends = 120 # minutes
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
        return 0, False, Lines
    PosOfT = Lines[1].find('t=')
    if PosOfT == -1:
        return 0, False, Lines
    TemporString = Lines[1].strip()[PosOfT + 2:]
    Result = float(TemporString) / 1000.0
    return Result, True, None



def Usage():
    print("Usage: main.py <sms_path>")


class TSensor:
    def __init__(self, TempPath, NameForSMS, MinPossibleTemperature):
        self.FirstTime = True
        self.MinT = 0
        self.MaxT = 0
        self.TimeOfMinT = ""
        self.TimeOfMaxT = ""
        self.TempPath = TempPath
        self.CurrentTemperature = 0
        self.ListOfTemperatures = list()
        self.NameForSMS = NameForSMS
        self.MinPossibleTemperature = MinPossibleTemperature


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
        self.ListOfTemperatures.append(ParseResult[0])
        if ParseResult[1] == False:
            print("Sensor " + self.NameForSMS + ": Parsing error!" + str(ParseResult[2]))
            SendSMS("ERROR AAAAA!!!!!!!!!!!!")
            return
        self.CurrentTemperature = ParseResult[0]
        self.UpdateStats(ParseResult[0])





AlreadySent = False
def SendSMSWhithStats():
    global AlreadySent
    global RegularSendingHour
    Now = datetime.datetime.now()
    #if (Now.hour == 15 or Now.hour == 17 or Now.hour == 19 or Now.hour == 21) and AlreadySent == False:
    if Now.hour == RegularSendingHour and AlreadySent == False:
        SendSMS(str(Sensor1.NameForSMS)       + " T = "    + str(Sensor1.CurrentTemperature) +
                ", Min = "                    + str(Sensor1.MinT) + "(" + str(Sensor1.TimeOfMinT) + ")" +
                ", Max = "                    + str(Sensor1.MaxT) + "(" + str(Sensor1.TimeOfMaxT) + ")."
                " " + str(Sensor2.NameForSMS) + " T = "   + str(Sensor2.CurrentTemperature) +
                ", Min = "                    + str(Sensor2.MinT) + "(" + str(Sensor2.TimeOfMinT) + ")" +
                ", Max = "                    + str(Sensor2.MaxT) + "(" + str(Sensor2.TimeOfMaxT) + ")"
                )
        FirstTime = True
        AlreadySent = True

    #if Now.hour == 16 or Now.hour == 18 or Now.hour == 20:
    if Now.hour == RegularSendingHour + 1:
        AlreadySent = False




def SendEmergencySMS():
    Text = ""
    if Sensor1.CurrentTemperature < Sensor1.MinPossibleTemperature:
        Text = Text + "Current Temperature at sensor " + Sensor1.NameForSMS  + ": " + str(Sensor1.CurrentTemperature) + ". MinPossibleTemperature: " + str(Sensor1.MinPossibleTemperature)
    if Sensor2.CurrentTemperature < Sensor2.MinPossibleTemperature:
        Text = Text + "Current Temperature at sensor " + Sensor2.NameForSMS  + ": " + str(Sensor2.CurrentTemperature) + ". MinPossibleTemperature: " + str(Sensor2.MinPossibleTemperature)
    if len(Text) > 0:
        SendSMS(Text)






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













import sys
import http.client
import xdrlib
import time

AUTH_SERVER = "sensorcloud.microstrain.com"

#samplerate types
HERTZ = 1
SECONDS = 0

def authenticate_key(device_id, key):
    """
    authenticate with sensorcloud and get the server and auth_key for all subsequent api requests
    """
    conn = http.client.HTTPSConnection(AUTH_SERVER)

    headers = {"Accept": "application/xdr"}
    url = "/SensorCloud/devices/%s/authenticate/?version=1&key=%s"%(device_id, key)

    print("authenticating...")
    conn.request('GET', url=url, headers=headers)
    response =conn.getresponse()
    print(response.status, response.reason)

    #if response is 200 ok then we can parse the response to get the auth token and server
    if response.status is http.client.OK:
        print("Credential are correct")

        #read the body of the response
        data = response.read()

        #response will be in xdr format. Create an XDR unpacker and extract the token and server as strings
        unpacker = xdrlib.Unpacker(data)
        auth_token = unpacker.unpack_string().decode('utf-8')
        server = unpacker.unpack_string().decode('utf-8')

        print("unpacked xdr.  server:%s  token:%s"%(server, auth_token))
        print("server type", type(server))

        return server, auth_token

def addSensor(server, auth_token, device_id, sensor_name, sensor_type="", sensor_label="", sensor_desc=""):
    """
    Add a sensor to the device. type, label, and description are optional.
    """

    conn = http.client.HTTPSConnection(server)

    url="/SensorCloud/devices/%s/sensors/%s/?version=1&auth_token=%s"%(device_id, sensor_name, auth_token)

    headers = {"Content-type" : "application/xdr"}

    #addSensor allows you to set the sensor type label and description.  All fileds are strings.
    #we need to pack these strings into an xdr structure
    packer = xdrlib.Packer()
    packer.pack_int(1)  #version 1
    packer.pack_string(sensor_type.encode('utf-8'))
    packer.pack_string(sensor_label.encode('utf-8'))
    packer.pack_string(sensor_desc.encode('utf-8'))
    data = packer.get_buffer()

    print("adding sensor...")
    conn.request('PUT', url=url, body=data, headers=headers)
    response =conn.getresponse()
    print(response.status , response.reason)

    #if response is 201 created then we know the sensor was added
    if response.status is http.client.CREATED:
        print("Sensor added")
    else:
        print("Error adding sensor. Error:", response.read())



def addChannel(server, auth_token, device_id, sensor_name, channel_name, channel_label="", channel_desc=""):
    """
    Add a channel to the sensor.  label and description are optional.
    """

    conn = http.client.HTTPSConnection(server)

    url="/SensorCloud/devices/%s/sensors/%s/channels/%s/?version=1&auth_token=%s"%(device_id, sensor_name, channel_name, auth_token)

    headers = {"Content-type" : "application/xdr"}

    #addChannel allows you to set the channel label and description.  All fileds are strings.
    #we need to pack these strings into an xdr structure
    packer = xdrlib.Packer()
    packer.pack_int(1)  #version 1
    packer.pack_string(channel_label.encode('utf-8'))
    packer.pack_string(channel_desc.encode('utf-8'))
    data = packer.get_buffer()

    print("adding channel...")
    conn.request('PUT', url=url, body=data, headers=headers)
    response =conn.getresponse()
    print(response.status , response.reason)

    #if response is 201 created then we know the channel was added
    if response.status is http.client.CREATED:
        print("Channel successfuly added")
    else:
        print("Error adding channel.  Error:", response.read())


class TOpenSensorData:
    Key = sys.argv[2]
    DeviceId = sys.argv[3]
    Server = None
    AuthToken = None
    LastUpload = datetime.datetime.now()
    LastAuthTime = datetime.datetime.now()
    PeriodOfUploading = 3 # in minutes, once in PeriodOfUploading minutes
    PeriodOfAuthing = 1 # in hours, once in PeriodOfAuthing hours


    def __init__(self):
        #first autheticate using the open api device serial and it's coresponding key
        #autheticate will return the server and an auth_token for all subsequent reguests
        self.Server, self.AuthToken = authenticate_key(self.DeviceId, self.Key)

        #add a new sensor to the device
        addSensor(self.Server, self.AuthToken, self.DeviceId, sensor_name="S1", sensor_desc="Tube (50a)")
        #now add a channel to the sensor
        addChannel(self.Server, self.AuthToken, self.DeviceId, sensor_name="S1", channel_name="Temp")

        #add a new sensor to the device
        addSensor(self.Server, self.AuthToken, self.DeviceId, sensor_name="S2", sensor_desc="Tube (dc2)")
        #now add a channel to the sensor
        addChannel(self.Server, self.AuthToken, self.DeviceId, sensor_name="S2", channel_name="Temp")

    def OnMeasurement(self):
        Now = datetime.datetime.now()
        if Now - self.LastUpload < datetime.timedelta(0, 0, 0, 0, self.PeriodOfUploading):
            return;
        if Now - self.LastAuthTime > datetime.timedelta(0, 0, 0, 0, 0, self.PeriodOfAuthing):
            self.Server, self.AuthToken = authenticate_key(self.DeviceId, self.Key)
            self.LastAuthTime = Now


        Seconds = (Now - self.LastUpload).total_seconds()
        self.UploadSensor(Sensor1, "S1", Seconds)
        self.UploadSensor(Sensor2, "S2", Seconds)

        self.LastUpload = Now


    def UploadSensor(self, Sensor, Name, Seconds):
        #print("OnMeasurement: Sensor: " + str(Sensor.ListOfTemperatures))
        Size = len(Sensor.ListOfTemperatures)
        k = Size / Seconds
        Freq = 1

        #print(Sensor.ListOfTemperatures)

        conn = http.client.HTTPSConnection(self.Server)
        url="/SensorCloud/devices/%s/sensors/%s/channels/%s/streams/timeseries/data/?version=1&auth_token=%s"%(self.DeviceId, Name, "Temp", self.AuthToken)
        #print(url)
        #print(self.Server)

        #we need to pack these strings into an xdr structure
        packer = xdrlib.Packer()
        packer.pack_int(1)  #version 1

        #set samplerate to 10 Hz
        packer.pack_enum(HERTZ)
        packer.pack_int(1)

        #Total number of datapoints.  6000 points is 10 minutes of data sampled at 10 Hz
        packer.pack_int(int(Seconds))

        #now pack each datapoint, we'll use a sin wave function to generate fake data.  we'll use the current time as the starting point
        timestamp_nanoseconds = int(time.time()*1000000000)  #start time in nanoseconds
        sampleInterval_nanoseconds = 1000000000  #number of nanoseconds between 2 datapoints when sampling at 1 Hz // 1s

        for i in range(0, int(Seconds)):
            packer.pack_hyper(timestamp_nanoseconds)
            packer.pack_float(Sensor.ListOfTemperatures[int(i * k)])  #generate value as a function of time
            #increment the timestamp for the next datapoint
            timestamp_nanoseconds += sampleInterval_nanoseconds

        data = packer.get_buffer()
        print("adding data...")
        headers = {"Content-type" : "application/xdr"}
        conn.request('POST', url=url, body=data, headers=headers)
        response =conn.getresponse()
        print(response.status , response.reason)
        #if response is 201 created then we know the channel was added
        if response.status is http.client.CREATED:
            print("data successfuly added")
        else:
            print("Error adding data.  Error:", response.read())

#        for i in range(0, int(Seconds)):
#            print(str(i) + ", index: " + str(int(i * k)) + ", " + str())

        Sensor.ListOfTemperatures = list()



# ======================================================== Main() ========================================================

# Parse command-line arguments:
#print("Number of arguments: " + str(len(sys.argv)) + " arguments.")
#print("Argument List: " + str(sys.argv))
ListenToSignal()

#if len(sys.argv) != 2:
#    Usage()
#    exit
SMSPassword = sys.argv[1]

Sensor1 = TSensor(TempPath1, "BottomTube", 15)
Sensor2 = TSensor(TempPath2, "Ambient", 8)
OpenSensorData = TOpenSensorData()

while True:
    time.sleep(1)
    Sensor1.ParseAndUpdate()
    Sensor2.ParseAndUpdate()
    SendEmergencySMS();
    SendSMSWhithStats()
    OpenSensorData.OnMeasurement()

Debug()
