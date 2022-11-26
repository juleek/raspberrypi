#!/usr/bin/python3
import abc
import devicedatum as dd
import typing as t
from logger import logger

class Consumer(abc.ABC):
    """
    See documentation for Ingest.

    The class corresponds to a single "sink":
    consume() is called with an instance of DeviceDatum, which is received from IoT device(s).
    """
    @abc.abstractmethod
    def consume(self, datum: dd.DeviceDatum) -> None:
        pass


class Ingest:
    """
    This is the main entry point for all incoming data from IoT device(s).
    The ctor gets a list of consumers (see above) and then notifies each of them on every DeviceDatum message.
    """
    def __init__(self, consumers: t.List[Consumer]):
        self.consumers = consumers

    def onDatum(self, datum: dd.DeviceDatum) -> None:
        logger.info(f'datum: {datum}')
        for consumer in self.consumers:
            consumer.consume(datum)
