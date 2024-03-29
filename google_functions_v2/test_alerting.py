import unittest
import bot_alerting as botalert
import sender as sender
import devicedatum as dd
import datetime as dt
import pytz
import unittest.mock as mc


class TestAlerting(unittest.TestCase):
    def setUp(self):
        self.name_to_min = {"Ambient": 6, "BottomTube": 12}
        self.sender = mc.Mock(spec=sender.Sender)
        self.alerting = botalert.Alerting(self.name_to_min, self.sender)
        self.tube_1 = "Ambient"
        self.tube_2 = "BottomTube"
        self.datum_below_threshold: dd.DeviceDatum = dd.DeviceDatum({self.tube_1: 4, self.tube_2: 11}, dt.datetime(2011, 11, 4, 0, 0, tzinfo=pytz.UTC), "")
        self.datum_above_threshold: dd.DeviceDatum = dd.DeviceDatum({self.tube_1: 7, self.tube_2: 13}, dt.datetime(2011, 11, 4, 0, 0, tzinfo=pytz.UTC), "")


    def test_alerting_sends_message_if_temp_below_threshold(self):
        self.alerting.consume(self.datum_below_threshold)
        self.sender.send_text.assert_called()
        self.assertTrue(self.sender.send_text.call_args[0][0])


    def test_alerting_does_not_send_message_if_temp_above_threshold(self):
        self.alerting.consume(self.datum_above_threshold)
        self.sender.send_text.assert_not_called()
