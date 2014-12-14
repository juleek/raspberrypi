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

#def authenticate_alternate(device_id, username, password):
#    """
#    authenticate with sensorcloud and get the server and auth_key for all subsequent api requests
#    """
#    conn = http.client.HTTPSConnection(AUTH_SERVER)

#    headers = {"Accept": "application/xdr"}
#    url = "/SensorCloud/devices/%s/authenticate/?version=1&username=%s&password=%s"%(device_id, username, password)

#    print("authenticating...")
#    conn.request('GET', url=url, headers=headers)
#    response =conn.getresponse()
#    print(response.status, response.reason)

#    #if response is 200 ok then we can parse the response to get the auth token and server
#    if response.status is http.client.OK:
#        print("Credential are correct")

#        #read the body of the response
#        data = response.read()

#        #response will be in xdr format. Create an XDR unpacker and extract the token and server as strings
#        unpacker = xdrlib.Unpacker(data)
#        auth_token = unpacker.unpack_string().decode('utf-8')
#        server = unpacker.unpack_string().decode('utf-8')

#        print("unpacked xdr.  server:%s  token:%s"%(server, auth_token))

#        return server, auth_token

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

def uploadSinWave(server, auth_token, device_id, sensor_name, channel_name):
    """
    Upload 10 minutes of 10 hz sin wave data.  use time.now() as the starting timestamp
    """

    conn = http.client.HTTPSConnection(server)

    url="/SensorCloud/devices/%s/sensors/%s/channels/%s/streams/timeseries/data/?version=1&auth_token=%s"%(device_id, sensor_name, channel_name, auth_token)

    #we need to pack these strings into an xdr structure
    packer = xdrlib.Packer()
    packer.pack_int(1)  #version 1

    #set samplerate to 10 Hz
    packer.pack_enum(HERTZ)
    packer.pack_int(10)


    #Total number of datapoints.  6000 points is 10 minutes of data sampled at 10 Hz
    POINTS = 5
    packer.pack_int(POINTS)

    print("generating data...")
    #now pack each datapoint, we'll use a sin wave function to generate fake data.  we'll use the current time as the starting point
    timestamp_nanoseconds = int(time.time()*1000000000)  #start time in nanoseconds
    sampleInterval_nanoseconds = 100000000  #number of nanoseconds between 2 datapoints when sampling at 10 Hz // 0.1s

    packer.pack_hyper(timestamp_nanoseconds)
    packer.pack_float(10)  #generate value as a function of time

    packer.pack_hyper(timestamp_nanoseconds + sampleInterval_nanoseconds)
    packer.pack_float(11)  #generate value as a function of time

    packer.pack_hyper(timestamp_nanoseconds + 10 * sampleInterval_nanoseconds)
    packer.pack_float(9)  #generate value as a function of time

    packer.pack_hyper(timestamp_nanoseconds + 20 * sampleInterval_nanoseconds)
    packer.pack_float(10)  #generate value as a function of time

    packer.pack_hyper(timestamp_nanoseconds + 30 * sampleInterval_nanoseconds)
    packer.pack_float(12)  #generate value as a function of time


#    for i in range(0,POINTS):
#        packer.pack_hyper(timestamp_nanoseconds)
#        packer.pack_float(math.sin(timestamp_nanoseconds/20000000000.0))  #generate value as a function of time

#        #increment the timestamp for the next datapoint
#        timestamp_nanoseconds += sampleInterval_nanoseconds

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

#def downloadData(server, auth_token, device_id, sensor_name, channel_name, startTime, endTime):
#    """
#    download the 10 minutes of data uploaded by uploadSinWave.
#    Returns an array of tuples, where each tuple is a timestamp and a value
#    """
#    conn = http.client.HTTPSConnection(server)

#    url = "/SensorCloud/devices/%s/sensors/%s/channels/%s/streams/timeseries/data/?version=1&auth_token=%s&starttime=%s&endtime=%s" % (device_id, sensor_name, channel_name, auth_token, startTime, endTime)
#    headers = {"Accept":"application/xdr"}
#    print("Downloading data...")
#    conn.request("GET", url=url, headers=headers)
#    response = conn.getresponse()
#    if response.status is http.client.OK:
#        print("Data retrieved")
#        data = []
#        unpacker = xdrlib.Unpacker(response.read())
#        while True:
#            try:
#                timestamp = unpacker.unpack_uhyper()
#                value = unpacker.unpack_float()
#                data.append((timestamp, value))
#            except Exception as e:
#                print(e)
#                break
#        return data
#    else:
#        print("Status: %s" % response.status)
#        print("Reason: %s" % response.reason)
#        return data



#info for API key authentication
Key = sys.argv[1]
DeviceId = sys.argv[2]

if __name__ == "__main__":
    #first autheticate using the open api device serial and it's coresponding key
    #autheticate will return the server and an auth_token for all subsequent reguests
    Server, AuthToken = authenticate_key(DeviceId, Key)

    #alternate method, uncomment to use
    #server, auth_token = authenticate_alternate(device_id, username, password)

    #add a new sensor to the device
    addSensor(Server, AuthToken, DeviceId, sensor_name="S1", sensor_desc="Tube (50a)")

#    #now add a channel to the sensor
    addChannel(Server, AuthToken, DeviceId, sensor_name="S1", channel_name="Temp")

#    #now add some data to the channel
    uploadSinWave(Server, AuthToken, DeviceId, sensor_name="S1", channel_name="temp")

#    #now download the channel data to an array
#    #we need to specify range, roughly we can guess that the data is from a minute ago up to 20 minutes from now (it's 10 minutes worth of data in our data generator above)
#    startTime = int(time.time()) - 60
#    endTime = startTime + 1200 #1200 = 20 minutes in seconds
#    #call download function, the time ranges are in nanoseconds
#    data = downloadData(server, auth_token, device_id, "S1", "temp", startTime*1000000000, endTime*1000000000)

#    print("Downloaded %s points" % len(data))
#    print("First point %s" % str(data[0]))
#    print("Last point %s" % str(data[-1]))
