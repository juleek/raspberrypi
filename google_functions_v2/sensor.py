#!/usr/bin/python3

from dataclasses import dataclass
import typing as t
import datetime as dt

@dataclass
class Sensor:
    """Class for keeping tube name with its temperatures and time when temperature was received.
        Class is storing data for creating a plot(graph) that we will use in a message."""
    temperatures: t.List[float]
    name: str
    timestamps: t.List[dt.datetime]


