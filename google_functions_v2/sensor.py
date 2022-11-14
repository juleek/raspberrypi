#!/usr/bin/python3

from dataclasses import dataclass
import typing as t
import datetime as dt

@dataclass
class Sensor:
    """
    This class keeps information that is relevant for a single sensor, in particular:
     * sensor's name
     * list of temperatures that was sent by the sensor
     * list of timestamps when temperature was received

    The data in this class is used to crate message and plot.
    """
    temperatures: t.List[float]
    name: str
    timestamps: t.List[dt.datetime]


