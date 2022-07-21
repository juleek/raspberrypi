import unittest
import unittest.mock as mc
import devicedatum as dd
import datetime as dt
import sensor_db as sdb


class TestSensorDBConsumer(unittest.TestCase):
    def test_write_calls_with_datum_elem(self):
        datum = dd.DeviceDatum({'example': 25.2}, dt.datetime.now(), " ")

        db = mc.Mock(spec=sdb.SensorsDB)

        consumer = sdb.Consumer(db)
        consumer.consume(datum)

        db.write.assert_called_with(datum)

