import unittest
import devicedatum as dd
import datetime as dt
import sensors_db_bg as sdbq
import sensor as sen
import pytz
import random
import string
import bigquerydb as bigqr

class TestSensorsDBBQ(unittest.TestCase):
    def setUp(self):
        s = ''.join(random.choice(string.ascii_lowercase) for i in range(10))
        self.db: sdbq.SensorsDBBQ = sdbq.SensorsDBBQ(bigqr.BigQueryDB(project="tarasovka", dataset_id="test", location="europe-west2"))
        self.tube_1 = "1"
        self.tube_2 = "2"
        self.good_datum_one_1: dd.DeviceDatum = dd.DeviceDatum({self.tube_1: 25.1}, dt.datetime(2011, 11, 4, 0, 0, tzinfo=pytz.UTC), "")
        self.good_datum_one_2: dd.DeviceDatum = dd.DeviceDatum({self.tube_1: 28.2}, dt.datetime(2011, 11, 5, 0, 0, tzinfo=pytz.UTC), "")
        self.good_datum_two: dd.DeviceDatum = dd.DeviceDatum({self.tube_1: 25.1, self.tube_2: 25.2}, dt.datetime(2011, 11, 6, 0, 0, tzinfo=pytz.UTC), "")
        self.err_datum_one_with_message: dd.DeviceDatum = dd.DeviceDatum({self.tube_2: 26.1}, dt.datetime(2012, 12, 5, 0, 0, tzinfo=pytz.UTC), "error")
        self.err_datum_one_with_message_2: dd.DeviceDatum = dd.DeviceDatum({self.tube_2: 26.1}, dt.datetime(2012, 12, 5, 0, 0, tzinfo=pytz.UTC), "error2")


    def test_empty_db_read_returns_empty(self):
        result = self.db.read_starting_from(dt.datetime.now())
        self.assertEqual(result, ([], set()))


    def test_read_with_period_greater_than_last_returns_empty(self):
        self.db.write(self.good_datum_one_1)
        self.db.write(self.good_datum_one_2)
        result = self.db.read_starting_from(self.good_datum_one_2.time + dt.timedelta(seconds=1))
        self.assertEqual(result, ([], set()))

    # def test_read_with_period_earlier_than_first(self):
    #     self.db.write(self.good_datum_one_1)
    #     self.db.write(self.good_datum_one_2)
    #
    #     result = self.db.read_starting_from(self.good_datum_one_1.time - dt.timedelta(seconds=1))
    #
    #     sensor: sen.Sensor = sen.Sensor(temperatures=[
    #         self.good_datum_one_1.name_to_temp[self.tube_1],
    #         self.good_datum_one_2.name_to_temp[self.tube_1]],
    #         name=self.tube_1,
    #         timestamps=[self.good_datum_one_1.time, self.good_datum_one_2.time])
    #
    #     self.assertEqual(result[0], [sensor])
    #     self.assertEqual(result[1], set())
    #
    #
    # def test_read_with_start_of_period_equal_to_one_of_elems(self):
    #     self.db.write(self.good_datum_one_1)
    #     self.db.write(self.good_datum_one_2)
    #     result = self.db.read_starting_from(self.good_datum_one_1.time)
    #
    #     sensor: sen.Sensor = sen.Sensor(temperatures=[
    #         self.good_datum_one_1.name_to_temp[self.tube_1],
    #         self.good_datum_one_2.name_to_temp[self.tube_1]],
    #         name=self.tube_1,
    #         timestamps=[self.good_datum_one_1.time, self.good_datum_one_2.time])
    #
    #     self.assertEqual(result[0], [sensor])
    #     self.assertEqual(result[1], set())


    # def test_read_with_start_of_period_between_two_elemes(self):
    #     self.db.write(self.good_datum_one_1)
    #     self.db.write(self.good_datum_one_2)
    #
    #     period_between_two_elem = self.good_datum_one_1.time + (self.good_datum_one_2.time - self.good_datum_one_1.time) / 2
    #
    #     result = self.db.read_starting_from(period_between_two_elem)
    #
    #     sensor: sen.Sensor = sen.Sensor(temperatures=[
    #         self.good_datum_one_2.name_to_temp[self.tube_1]],
    #         name=self.tube_1,
    #         timestamps=[self.good_datum_one_2.time])
    #
    #     self.assertEqual(result[0], [sensor])
    #     self.assertEqual(result[1], set())
    #
    #
    # def test_read_one_empty_name_to_temp(self):
    #     datum: dd.DeviceDatum = dd.DeviceDatum({}, dt.datetime.now(), "")
    #
    #     self.db.write(datum)
    #
    #     result = self.db.read_starting_from(dt.datetime.now())
    #     self.assertEqual(result, ([], set()))
    #
    # def test_read_two_equal_non_empty_name_to_temp(self):
    #     self.db.write(self.good_datum_one_1)
    #     self.db.write(self.good_datum_one_2)
    #
    #     result = self.db.read_starting_from(self.good_datum_one_1.time - dt.timedelta(seconds=1))
    #
    #     sensor: sen.Sensor = sen.Sensor(temperatures=[
    #         self.good_datum_one_1.name_to_temp[self.tube_1],
    #         self.good_datum_one_2.name_to_temp[self.tube_1]],
    #         name=self.tube_1,
    #         timestamps=[self.good_datum_one_1.time, self.good_datum_one_2.time])
    #
    #     self.assertEqual(result[0], [sensor])
    #     self.assertEqual(result[1], set())
    #
    #
    # def test_read_one_empty_one_non_empty(self):
    #     datum_with_empty_dict: dd.DeviceDatum = dd.DeviceDatum({}, dt.datetime(2011, 11, 4, 0, 0), "")
    #     self.db.write(self.good_datum_one_1)
    #     self.db.write(datum_with_empty_dict)
    #
    #     result = self.db.read_starting_from(self.good_datum_one_1.time - dt.timedelta(seconds=1))
    #
    #     sensor: sen.Sensor = sen.Sensor(temperatures=[
    #         self.good_datum_one_1.name_to_temp[self.tube_1]],
    #         name=self.tube_1,
    #         timestamps=[self.good_datum_one_1.time])
    #
    #     self.assertEqual(result[0], [sensor])
    #     self.assertEqual(result[1], set())
    #
    #
    # def test_read_two_partially_equal_name_to_temp(self):
    #     self.db.write(self.good_datum_one_1)
    #     self.db.write(self.good_datum_two)
    #
    #     result = self.db.read_starting_from(self.good_datum_one_1.time - dt.timedelta(seconds=1))
    #
    #     sensor_tube_1: sen.Sensor = sen.Sensor(temperatures=[
    #         self.good_datum_one_1.name_to_temp[self.tube_1],
    #         self.good_datum_two.name_to_temp[self.tube_1]],
    #         name=self.tube_1,
    #         timestamps=[self.good_datum_one_1.time, self.good_datum_two.time])
    #
    #     sensor_tube_2: sen.Sensor = sen.Sensor(temperatures=[
    #         self.good_datum_two.name_to_temp[self.tube_2]],
    #         name=self.tube_2,
    #         timestamps=[self.good_datum_two.time])
    #
    #
    #     self.assertEqual(result[0], [sensor_tube_1, sensor_tube_2])
    #     self.assertEqual(result[1], set())

    #
    # def test_read_two_datum_with_equal_err_messages(self):
    #     self.db.write(self.err_datum_one_with_message)
    #     self.db.write(self.err_datum_one_with_message)
    #
    #     result = self.db.read_starting_from(self.err_datum_one_with_message.time - dt.timedelta(seconds=1))
    #
    #     sensor: sen.Sensor = sen.Sensor(temperatures=[
    #         self.err_datum_one_with_message.name_to_temp[self.tube_2],
    #         self.err_datum_one_with_message.name_to_temp[self.tube_2]],
    #         name=self.tube_2,
    #         timestamps=[self.err_datum_one_with_message.time, self.err_datum_one_with_message.time])
    #
    #     self.assertEqual(result[0], [sensor])
    #     self.assertEqual(result[1], {self.err_datum_one_with_message.error_msg})
    #
    #
    # def test_read_two_datum_with_different_err_messages(self):
    #     self.db.write(self.err_datum_one_with_message)
    #     self.db.write(self.err_datum_one_with_message_2)
    #
    #     result = self.db.read_starting_from(self.err_datum_one_with_message.time - dt.timedelta(seconds=1))
    #
    #     sensor: sen.Sensor = sen.Sensor(temperatures=[
    #         self.err_datum_one_with_message.name_to_temp[self.tube_2],
    #         self.err_datum_one_with_message_2.name_to_temp[self.tube_2]],
    #         name=self.tube_2,
    #         timestamps=[self.err_datum_one_with_message.time, self.err_datum_one_with_message_2.time])
    #
    #     self.assertEqual(result[0], [sensor])
    #     self.assertEqual(result[1], {self.err_datum_one_with_message.error_msg, self.err_datum_one_with_message_2.error_msg})
    #
    #
    # def test_delete_before_earlier_than_first(self):
    #     self.db.write(self.good_datum_one_1)
    #     self.db.write(self.good_datum_one_2)
    #
    #     self.db.delete_before(self.good_datum_one_1.time - dt.timedelta(seconds=1))
    #
    #     result = self.db.read_starting_from(self.good_datum_one_1.time - dt.timedelta(seconds=1))
    #
    #     sensor: sen.Sensor = sen.Sensor(temperatures=[
    #         self.good_datum_one_1.name_to_temp[self.tube_1],
    #         self.good_datum_one_2.name_to_temp[self.tube_1]],
    #         name=self.tube_1,
    #         timestamps=[self.good_datum_one_1.time, self.good_datum_one_2.time])
    #
    #     self.assertEqual(result[0], [sensor])
    #     self.assertEqual(result[1], set())
    #
    #
    # def test_delete_before_greater_than_last(self):
    #     print(self.db.table)
    #     self.db.write(self.good_datum_one_1)
    #     self.db.write(self.good_datum_one_2)

        # self.db.delete_before(self.good_datum_one_2.time + dt.timedelta(seconds=1))
        #
        # result = self.db.read_starting_from(self.good_datum_one_1.time - dt.timedelta(seconds=1))
        #
        # print(f'Actual = {result}')
        # self.assertEqual(result, ([], set()))
    #
    #
    # def test_delete_before_date_between_two_elemes(self):
    #     self.db.write(self.good_datum_one_1)
    #     self.db.write(self.good_datum_one_2)
    #
    #     date_between_two_elem = self.good_datum_one_1.time + (self.good_datum_one_2.time - self.good_datum_one_1.time) / 2
    #
    #     self.db.delete_before(date_between_two_elem)
    #
    #     result = self.db.read_starting_from(self.good_datum_one_1.time - dt.timedelta(seconds=1))
    #
    #     sensor: sen.Sensor = sen.Sensor(temperatures=[
    #         self.good_datum_one_2.name_to_temp[self.tube_1]],
    #         name=self.tube_1,
    #         timestamps=[self.good_datum_one_2.time])
    #
    #     self.assertEqual(result[0], [sensor])
    #     self.assertEqual(result[1], set())
