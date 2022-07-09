import unittest
import unittest.mock as mc
import ingest as ing
import typing as t
import devicedatum as dd
import datetime as dt


class TestIngest(unittest.TestCase):
    def test_onDatum_calls_consume_elems(self):
        for num_of_consumers in [0, 1, 2, 5]:
            device_datum = dd.DeviceDatum({'example': 25.2}, dt.datetime.now(), " ")
            consumers: t.List = []
            for i in range(num_of_consumers):
                consumers.append(mc.Mock(spec=ing.Consumer))

            ingest = ing.Ingest(consumers)
            ingest.onDatum(device_datum)

            for c in consumers:
                c.consume.assert_called_with(device_datum)

