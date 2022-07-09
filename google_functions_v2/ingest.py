#!/usr/bin/python3
import abc
import devicedatum as dd
import typing as t

class Consumer(abc.ABC):
    @abc.abstractmethod
    def consume(self, datum: dd.DeviceDatum) -> None:
        pass


class Ingest:
    def __init__(self, consumers: t.List[Consumer]):
        self.consumers = consumers

    def onDatum(self, datum: dd.DeviceDatum) -> None:
        for consumer in self.consumers:
            consumer.consume(datum)
