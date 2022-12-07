import sensor_db as sdb
import typing as t
from devicedatum import DeviceDatum
import datetime as dt
import sensor as sen
from collections import defaultdict
import bisect as bs

class SensorsDBFake(sdb.SensorsDB):
    def __init__(self):
        self.data: t.List[DeviceDatum] = []

    def write(self,  datum: DeviceDatum) -> None:
        self.data.append(datum)

    def read_starting_from(self, date: dt.datetime) -> t.Tuple[t.List[sen.Sensor], t.Set[str]]:
        ind: int = bs.bisect_left(self.data, date, key=lambda dd: dd.time)
        name_to_sensor_temp: t.Dict[str, sen.Sensor] = defaultdict(lambda: sen.Sensor(temperatures=[], name="", timestamps=[]))
        messages: t.Set[str] = set()
        for i in range(ind, len(self.data)):
            for tube_name, temp in self.data[i].name_to_temp.items():
                name_to_sensor_temp[tube_name].temperatures.append(temp),
                name_to_sensor_temp[tube_name].timestamps.append(self.data[i].time),
                name_to_sensor_temp[tube_name].name = tube_name
                if self.data[i].error_msg:
                    messages.add(self.data[i].error_msg)
        sensors: t.List[sen.Sensor] = list(name_to_sensor_temp.values())
        return sensors, messages


    def delete_before(self, date: dt.datetime) -> None:
        del_before_ind: int = bs.bisect(self.data, date, key=lambda dd: dd.time)
        del self.data[:del_before_ind]

    def read_last_result(self) -> t.Tuple[t.List[sen.Sensor], t.Set[str]]:
        pass

