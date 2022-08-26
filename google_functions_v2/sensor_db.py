#!/usr/bin/python3
import abc
from devicedatum import DeviceDatum
from sensor import Sensor
import ingest
import datetime as dt
import typing as t

class SensorsDB(abc.ABC):
    @abc.abstractmethod
    def write(self,  datum: DeviceDatum) -> None:
        pass

    @abc.abstractmethod
    def read_starting_from(self, date: dt.datetime) -> t.Tuple[t.List[Sensor], t.Set[str]]:
        pass


    def read_for_period(self, period: dt.timedelta) -> t.Tuple[t.List[Sensor], t.Set[str]]:
        return self.read_starting_from(dt.datetime.now() - period)


    @abc.abstractmethod
    def delete_before(self, date: dt.datetime) -> None:
        pass

    def delete(self, older_than: dt.timedelta) -> None:
        return self.delete_before(dt.datetime.now() - older_than)



class Consumer(ingest.Consumer):
    def __init__(self, db: SensorsDB):
        self.db = db

    def consume(self, datum: DeviceDatum):
        self.db.write(datum)