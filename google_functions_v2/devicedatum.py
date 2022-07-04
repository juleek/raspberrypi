#!/usr/bin/python3
from dataclasses import dataclass
import datetime as dt
import typing as t

@dataclass
class DeviceDatum:
    name_to_temp: t.Dict[str, float]
    time: dt.datetime
    error_msg: str

