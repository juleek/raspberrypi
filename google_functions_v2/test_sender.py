import unittest
import telegram_sender as ts


class TestTelegramSender(unittest.TestCase):
    def test_get_chat_id_get_chatid_from_str(self):
        result = ts.get_chatid_from_str('{"update_id":802398224, "message":{"message_id":23,"from":{"id":1111,"first_name":"Y","last_name":"Y", "username":"y","language_code":"en"},"chat":{"id":111,"title":"title","type": "group"},"date":1660129735}}')
        self.assertEqual(result, 111)
