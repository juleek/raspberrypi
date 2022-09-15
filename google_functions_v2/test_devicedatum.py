#!/usr/bin/python3

import unittest
import devicedatum as dd
import datetime as dt


class TestDeviceDatumFromJason(unittest.TestCase):
    def test_str_json_parse_to_datum_with_error(self):
        actual = dd.DeviceDatum.from_json('{"name_to_temp": {"Ambient":20.2, "Bottom":3.5}, "time":"1999-05-25 02:35:05.523000", "error_msg":"error"}')
        expected = dd.DeviceDatum(name_to_temp={'Ambient': 20.2, 'Bottom': 3.5},
                                 time=dt.datetime(1999, 5, 25, 2, 35, 5, 523000),
                                 error_msg="error")
        self.assertEqual(actual, expected)

    def test_str_json_parse_to_datum_without_error(self):
        actual = dd.DeviceDatum.from_json('{"name_to_temp": {"Ambient":20.2, "Bottom":3.5}, "time":"1999-05-25 02:35:05.523000", "error_msg":""}')
        expected = dd.DeviceDatum(name_to_temp={'Ambient': 20.2, 'Bottom': 3.5},
                                  time=dt.datetime(1999, 5, 25, 2, 35, 5, 523000),
                                  error_msg="")
        self.assertEqual(actual, expected)

    def test_str_json_parse_to_datum_with_empty_name_to_temp(self):
        actual = dd.DeviceDatum.from_json('{"name_to_temp": {}, "time":"1999-05-25 02:35:05.523000", "error_msg":"error"}')
        expected = dd.DeviceDatum(name_to_temp={},
                                  time=dt.datetime(1999, 5, 25, 2, 35, 5, 523000),
                                  error_msg="error")
        self.assertEqual(actual, expected)
