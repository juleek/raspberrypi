#!/usr/bin/python3

from dataclasses import dataclass
import typing as t
import datetime as dt

@dataclass
class Sensor:
    """
    This class is needed is keeping information that relevant for a sensor, in particular:
     * sensor's name
     * list of temperatures that was sent by the sensor
     * list of datetime when temperature was received

    The data in this class is used in creating msg and plot.
    """
    temperatures: t.List[float]
    name: str
    timestamps: t.List[dt.datetime]


