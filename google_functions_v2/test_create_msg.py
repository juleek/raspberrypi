import unittest
import typing as t
import sensor as sen
import datetime as dt
import bot_notifier as botnotifier

class TestCreateMsg(unittest.TestCase):
    def setUp(self):
        self.tube_1: str = "example"
        self.tube_2: str = "test"
        self.error_msg_1: str = 'error'
        self.error_msg_2: str = 'error2'
        self.temperatures_1: t.List[float] = [25.1, 28.2, 26.2]
        self.temperatures_2: t.List[float] = [6.2, 10.3, 14.2]
        s1: sen.Sensor = sen.Sensor(temperatures=self.temperatures_1,
                                    name=self.tube_1,
                                    timestamps=[dt.datetime(2011, 11, 4, 0, 0, tzinfo=dt.timezone.utc)])
        s2: sen.Sensor = sen.Sensor(temperatures=self.temperatures_2,
                                    name=self.tube_2,
                                    timestamps=[dt.datetime(2011, 11, 4, 0, 0, tzinfo=dt.timezone.utc)])
        self.sensors: t.List[sen.Sensor] = [s1, s2]
        self.with_error_msgs: t.Set[str] = {self.error_msg_1, self.error_msg_2}

    def check_sensor_part(self, actual: str):
        self.assertIn(self.tube_1, actual)
        self.assertIn(self.tube_2, actual)
        self.assertIn(str(min(self.temperatures_1)), actual)
        self.assertIn(str(max(self.temperatures_2)), actual)

    def check_error_part(self, actual: str):
        self.assertIn(self.error_msg_1, actual)
        self.assertIn(self.error_msg_2, actual)

    def test_create_msg_with_empty_args_return_empty(self):
        actual: str = botnotifier.create_msg([], set())
        self.assertEqual("", actual)

    def test_create_msg_with_empty_sensor_with_error_return_error_msg(self):
        actual: str = botnotifier.create_msg([], self.with_error_msgs)
        self.check_error_part(actual)

    def test_create_msg_with_sensors_and_empty_error_msg(self):
        actual: str = botnotifier.create_msg(self.sensors, set())
        self.check_sensor_part(actual)

    def test_create_msg_with_sensors_and_error_msg(self):
        actual: str = botnotifier.create_msg(self.sensors, self.with_error_msgs)
        self.check_sensor_part(actual)
        self.check_error_part(actual)

