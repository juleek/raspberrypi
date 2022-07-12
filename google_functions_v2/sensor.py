#!/usr/bin/python3

from dataclasses import dataclass
import typing as t
import datetime as dt

@dataclass
class Sensor:
    temperatures: t.List[float]
    name: str
    timestamps: t.List[dt.datetime]


