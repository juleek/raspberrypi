#!/usr/bin/python3

import unittest
import devicedatum as dd
import datetime as dt
import pytz


class TestDeviceDatum(unittest.TestCase):
    def test_from_json_with_error_message(self):
        actual = dd.DeviceDatum.from_json('{"name_to_temp": {"Ambient":20.2, "Bottom":3.5}, "time":"1999-05-25T02:35:05.523000+00:00", "error_msg":"error"}')
        expected = dd.DeviceDatum(name_to_temp={'Ambient': 20.2, 'Bottom': 3.5},
                                 time=dt.datetime(1999, 5, 25, 2, 35, 5, 523000, tzinfo=pytz.UTC),
                                 error_msg="error")
        self.assertEqual(actual, expected)

    def test_from_json_without_error(self):
        actual = dd.DeviceDatum.from_json('{"name_to_temp": {"Ambient":20.2, "Bottom":3.5}, "time":"1999-05-25T02:35:05.523000+00:00"}')
        expected = dd.DeviceDatum(name_to_temp={'Ambient': 20.2, 'Bottom': 3.5},
                                  time=dt.datetime(1999, 5, 25, 2, 35, 5, 523000, tzinfo=pytz.UTC),
                                  error_msg="")
        self.assertEqual(actual, expected)

    def test_from_json_with_empty_name_to_temp(self):
        actual = dd.DeviceDatum.from_json('{"name_to_temp": {}, "time":"1999-05-25T02:35:05.523000+00:00", "error_msg":"error"}')
        expected = dd.DeviceDatum(name_to_temp={},
                                  time=dt.datetime(1999, 5, 25, 2, 35, 5, 523000, tzinfo=pytz.UTC),
                                  error_msg="error")
        self.assertEqual(actual, expected)
