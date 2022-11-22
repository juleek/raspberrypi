#!/usr/bin/python3
from dataclasses import dataclass
import datetime as dt
import typing as t
import json

@dataclass
class DeviceDatum:
    """
    The is the main struct that corresponds to a single "measurement" (data-point received from IoT device).
    """
    name_to_temp: t.Dict[str, float]
    time: dt.datetime
    error_msg: str

    @staticmethod
    def from_json(str_json: str) -> 'DeviceDatum':
        msg = json.loads(str_json)
        time = dt.datetime.fromisoformat(msg['Time'])
        return DeviceDatum(
            name_to_temp=msg['NameToTemp'],
            time=time,
            error_msg=msg['ErrorString'] if "ErrorString" in msg else "")
