#!/usr/bin/python3
import abc
from devicedatum import DeviceDatum
import typing as t

class Consumer(abc.ABC):
    @abc.abstractmethod
    def consume(self, datum: DeviceDatum) -> None:
        pass


class Ingest:
    def __init__(self, consumers: t.List[Consumer]):
        self.consumers = consumers

    def onDatum(self, datum: DeviceDatum) -> None:
        for consumer in self.consumers:
            consumer.consume(datum)
