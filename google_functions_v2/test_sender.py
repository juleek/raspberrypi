import unittest
import telegram_sender as ts


class TestTelegramSender(unittest.TestCase):
    def test_get_chatid_from_str(self):
        result_message = ts.get_chatid_from_str('{"update_id":319349776,"message":{"message_id":309,"from":{"id":123,"is_bot":false,"first_name":"w","last_name":"w"},"chat":{"id":6666,"title":"title","type":"group"},"date":1673276449}}')
        result_edited_message = ts.get_chatid_from_str('{"update_id":319349776,"edited_message":{"message_id":309,"from":{"id":123,"is_bot":false,"first_name":"w","last_name":"w"},"chat":{"id":777,"title":"title","type":"group"},"date":1673276449}}')
        self.assertEqual(result_message, 6666)
        self.assertEqual(result_edited_message, 777)
