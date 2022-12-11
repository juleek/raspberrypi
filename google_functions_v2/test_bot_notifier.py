import unittest
import typing as t
import sensor as sen
import datetime as dt
import bot_notifier as botnotifier
import unittest.mock as mc
import sender
import sensors_db_bg as sdbq
import chat_id_db as chidb

tube_1: str = "example"
tube_2: str = "test"
error_msg_1: str = 'error'
error_msg_2: str = 'error2'

def check_sensor_part(self, actual: str):
    self.assertIn(tube_1, actual)
    self.assertIn(tube_2, actual)
    self.assertIn(str(min(self.temperatures_1)), actual)
    self.assertIn(str(max(self.temperatures_2)), actual)


def check_error_part(self, actual: str):
    self.assertIn(error_msg_1, actual)
    self.assertIn(error_msg_2, actual)

class TestCreateMsg(unittest.TestCase):
    def setUp(self):
        self.temperatures_1: t.List[float] = [25.1, 28.2, 26.2]
        self.temperatures_2: t.List[float] = [6.2, 10.3, 14.2]
        s1: sen.Sensor = sen.Sensor(temperatures=self.temperatures_1,
                                    name=tube_1,
                                    timestamps=[dt.datetime(2011, 11, 4, 0, 0, tzinfo=dt.timezone.utc)])
        s2: sen.Sensor = sen.Sensor(temperatures=self.temperatures_2,
                                    name=tube_2,
                                    timestamps=[dt.datetime(2011, 11, 4, 0, 0, tzinfo=dt.timezone.utc)])
        self.sensors: t.List[sen.Sensor] = [s1, s2]
        self.with_error_msgs: t.Set[str] = {error_msg_1, error_msg_2}

    def test_empty_args_return_empty(self):
        actual: str = botnotifier.create_msg([], set())
        self.assertEqual("", actual)

    def test_empty_sensor_with_error_return_error_msg(self):
        actual: str = botnotifier.create_msg([], self.with_error_msgs)
        check_error_part(self, actual)

    def test_sensors_and_empty_error_msg_return_msg_with_sensors(self):
        actual: str = botnotifier.create_msg(self.sensors, set())
        check_sensor_part(self, actual)

    def test_and_error_msg_return_msg_with_sensors_and_errors(self):
        actual: str = botnotifier.create_msg(self.sensors, self.with_error_msgs)
        check_sensor_part(self, actual)
        check_error_part(self, actual)


class TestCreateMsgWithCurrTemp(unittest.TestCase):
    def setUp(self):
        self.temperatures_1: t.List[float] = [26.2]
        self.temperatures_2: t.List[float] = [14.2]
        s1: sen.Sensor = sen.Sensor(temperatures=self.temperatures_1,
                                    name=tube_1,
                                    timestamps=[dt.datetime(2022, 11, 4, 0, 0, tzinfo=dt.timezone.utc)])
        s2: sen.Sensor = sen.Sensor(temperatures=self.temperatures_2,
                                    name=tube_2,
                                    timestamps=[dt.datetime(2022, 11, 4, 0, 0, tzinfo=dt.timezone.utc)])

        s3: sen.Sensor = sen.Sensor(temperatures=self.temperatures_2,
                                    name=tube_2,
                                    timestamps=[dt.datetime(2022, 11, 5, 0, 0, tzinfo=dt.timezone.utc)])
        s4: sen.Sensor = sen.Sensor(temperatures=self.temperatures_2,
                                    name=tube_1,
                                    timestamps=[dt.datetime(2022, 11, 5, 0, 0, tzinfo=dt.timezone.utc)])
        self.sensors: t.List[sen.Sensor] = [s1, s2]
        self.sensors_four: t.List[sen.Sensor] = [s1, s2, s3, s4]
        self.with_error_msgs: t.Set[str] = {error_msg_1, error_msg_2}


    def test_empty_args_return_empty(self):
        actual: str = botnotifier.create_msg_with_current_temp([], set())
        self.assertEqual("", actual)

    def test_empty_sensor_with_error_return_error_msg(self):
        actual: str = botnotifier.create_msg_with_current_temp([], self.with_error_msgs)
        check_error_part(self, actual)

    def test_sensors_and_empty_error_msg_return_msg_with_sensors(self):
        actual: str = botnotifier.create_msg_with_current_temp(self.sensors, set())
        check_sensor_part(self, actual)

    def test_sensors_and_error_msg_return_msg_with_sensors_and_errors(self):
        actual: str = botnotifier.create_msg_with_current_temp(self.sensors, self.with_error_msgs)
        check_sensor_part(self, actual)
        check_error_part(self, actual)

    def test_print_msg(self):
        actual = botnotifier.create_msg_with_current_temp(self.sensors_four, self.with_error_msgs)
        print(actual)


class TestSendCurrentTempMsg(unittest.TestCase):
    def setUp(self):
        self.sender = mc.Mock(spec=sender.Sender)


    def test_send_curr_temp_call_sender(self):
        s1: sen.Sensor = sen.Sensor(temperatures=[26.2],
                                    name="example",
                                    timestamps=[dt.datetime(2022, 11, 4, 0, 0, tzinfo=dt.timezone.utc)])
        s2: sen.Sensor = sen.Sensor(temperatures=[14.2],
                                    name="test1",
                                    timestamps=[dt.datetime(2022, 11, 4, 0, 0, tzinfo=dt.timezone.utc)])
        sensorsdbbq = mc.Mock(spec=sdbq.SensorsDBBQ)
        sensorsdbbq.read_last_result = mc.Mock(spec=sdbq.SensorsDBBQ.read_last_result, return_value=([s1, s2], set()))
        botnotifier.send_current_temperature(sensorsdbbq, self.sender)
        self.sender.send_text.assert_called()



class TestDispatchCommand(unittest.TestCase):
    def setUp(self):
        self.sensors_db = mc.Mock(spec=sdbq.SensorsDBBQ)
        self.chat_id_db = mc.Mock(spec=chidb.ChatIdDB)
        self.sender = mc.Mock(spec=sender.Sender)
        self.chat_id_unknown: int = 11111
        self.chat_id_known: int = 22222

    def test_ask_to_add_is_called_when_chatid_is_unknown_without_cmd(self):
        jsn1: t.Dict = {
            "update_id": 2222222,
            "message": {
                "message_id": 23,
                "from": {
                    "id": 1111,
                    "first_name": "asdf",
                    "last_name": "asdf",
                    "username": "asdf",
                    "language_code": "en"
                },
                "chat": {
                    "id": -111111,
                    "title": "asdf",
                    "type": "group"
                },
                "date": 111111
            }
        }

        jsn2: t.Dict = {
            "update_id": 2222222,
            "message": {
                "message_id": 23,
                "from": {
                    "id": 1111,
                    "first_name": "asdf",
                    "last_name": "asdf",
                    "username": "asdf",
                    "language_code": "en"
                },
                "chat": {
                    "id": -111111,
                    "title": "asdf",
                    "type": "group"
                },
                "date": 111111
            }
        }


        for jsn in [jsn1, jsn2]:
            chat_id_db = mc.Mock(spec=chidb.ChatIdDB)
            chat_id_db.exists = mc.Mock(spec=chidb.ChatIdDB.exists, return_value=False)
            botnotifier.dispatch_command(jsn, self.chat_id_unknown, chat_id_db, self.sensors_db, self.sender)
            chat_id_db.ask_to_add.assert_called()


    @mc.patch('bot_notifier.send_current_temperature')
    def test_send_current_temperature_is_called_if_gettemp_command_from_known_chat(self, mock_curr_temp_msg):
        jsn: t.Dict = {
          "update_id": 1111,
          "message": {
            "message_id": 27,
            "from": {
              "id": 1111,
              "is_bot": False,
              "first_name": "asdf",
              "last_name": "asdf"
            },
            "chat": {
              "id": -22222,
              "title": "asdf",
              "type": "group",
              "all_members_are_administrators": True
            },
            "date": 22222,
            "text": "/gettemp",
            "entities": [
              {
                "offset": 0,
                "length": 5,
                "type": "bot_command"
              }
            ]
          }
        }

        self.chat_id_db.exists = mc.Mock(return_value=True)
        botnotifier.dispatch_command(jsn, self.chat_id_known, self.chat_id_db, self.sensors_db, self.sender)
        mock_curr_temp_msg.assert_called()


    def test_sender_is_not_called_when_command_is_uknown_from_known_chat(self):
        jsn: t. Dict = {
          "update_id": 1111,
          "message": {
            "message_id": 27,
            "from": {
              "id": 1111,
              "is_bot": False,
              "first_name": "asdf",
              "last_name": "asdf"
            },
            "chat": {
              "id": -22222,
              "title": "asdf",
              "type": "group",
              "all_members_are_administrators": True
            },
            "date": 22222,
            "text": "/test",
            "entities": [
              {
                "offset": 0,
                "length": 5,
                "type": "bot_command"
              }
            ]
          }
        }
        self.chat_id_db.exists = mc.Mock(return_value=True)
        botnotifier.dispatch_command(jsn, self.chat_id_known, self.chat_id_db, self.sensors_db, self.sender)
        self.sender.assert_not_called()
