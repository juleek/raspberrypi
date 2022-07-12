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
    def read(self, period: dt.timedelta) -> t.Tuple[t.List[Sensor], t.Set[str]]:
        pass

    @abc.abstractmethod
    def delete(self, older_than: dt.timedelta) -> None:
        pass


class Consumer(ingest.Consumer):
    def __init__(self, db: SensorsDB):
        self.db = db

    def consume(self, datum: DeviceDatum):
        self.db.write(datum)
