import unittest
import telegram_sender as ts


class TestTelegramSender(unittest.TestCase):
    def test_get_chat_id_with_update_with_chat_id(self):
        result = ts.get_chat_id_from_update_msg('{"update_id":802398224, "message":{"message_id":23,"from":{"id":124375341,"first_name":"Yulia","last_name":"Yulia", "username":"lon_yul","language_code":"en"},"chat":{"id":-748244195,"title":"TEST Group WITH Test1","type": "group"},"date":1660129735}}')
        self.assertEqual(result, -748244195)
