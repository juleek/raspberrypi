#!/usr/bin/python3
import abc
from devicedatum import DeviceDatum
from sensor import Sensor
import ingest
import datetime as dt
import typing as t

class SensorsDB(abc.ABC):
    """
    This is an abstract class that is responsible for:
    * writing DeviceDatum to DB,
    * reading data from DB for the provided period and return list of Sensors and error_messages,
    * deleting data from DB (if needed).
    """
    @abc.abstractmethod
    def write(self,  datum: DeviceDatum) -> None:
        pass

    @abc.abstractmethod
    def read_starting_from(self, date: dt.datetime) -> t.Tuple[t.List[Sensor], t.Set[str]]:
        pass

    def read_for_period(self, period: dt.timedelta) -> t.Tuple[t.List[Sensor], t.Set[str]]:
        return self.read_starting_from(dt.datetime.now() - period)

    @abc.abstractmethod
    def read_last_result(self) -> t.Tuple[t.List[Sensor], t.Set[str]]:
        pass


    @abc.abstractmethod
    def delete_before(self, date: dt.datetime) -> None:
        pass

    def delete(self, older_than: dt.timedelta) -> None:
        return self.delete_before(dt.datetime.now() - older_than)


class DBConsumer(ingest.Consumer):
    """
    This is a database writing implementation of the abstract class Consumer:
    on every DeviceDatum it writes DeviceDatum to DB.
    """
    def __init__(self, db: SensorsDB):
        self.db = db

    def consume(self, datum: DeviceDatum):
        self.db.write(datum)
