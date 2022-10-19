#!/usr/bin/python3
from dataclasses import dataclass
import datetime as dt
import typing as t
import json

@dataclass
class DeviceDatum:
    """
    The class is responsible for:
    * converting data's format that received from IoT device
    * keeping data in a new format "DeviceDatum". This format is convenient for filling the tables.
    """
    name_to_temp: t.Dict[str, float]
    time: dt.datetime
    error_msg: str

    @staticmethod
    def from_json(str_json: str) -> 'DeviceDatum':
        msg = json.loads(str_json)
        time = dt.datetime.fromisoformat(msg['time'])
        return DeviceDatum(
            name_to_temp=msg['name_to_temp'],
            time=time,
            error_msg=msg['error_msg'] if "error_msg" in msg else "")
