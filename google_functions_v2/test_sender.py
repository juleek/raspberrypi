import unittest
import telegram_sender as ts


class TestTelegramSender(unittest.TestCase):
    def test_get_chatid_from_str(self):
        result = ts.get_chatid_from_str('{"update_id":802398224, "message":{"message_id":23,"from":{"id":llll,"first_name":"Y","last_name":"Y", "username":"y","language_code":"en"},"chat":{"id":111,"title":"title","type": "group"},"date":1660129735}}')
        self.assertEqual(result, 111)

    def test_get_chatid_from_str_edited_message(self):
        result = ts.get_chatid_from_str('{"update_id":802398224, "edited_message":{"message_id":23,"from":{"id":llll,"first_name":"Y","last_name":"Y", "username":"y","language_code":"en"},"chat":{"id":222,"title":"title","type": "group"},"date":1660129735}}')
        self.assertEqual(result, 222)
