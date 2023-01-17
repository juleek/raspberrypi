import unittest
import telegram_sender as ts


class TestTelegramSender(unittest.TestCase):
    def test_get_chatid_from_str(self):
        jsn_with_msg = '{"update_id":319349776,"message":{"message_id":309,"from":{"id":123,"is_bot":false,"first_name":"w","last_name":"w"},"chat":{"id":6666,"title":"title","type":"group"},"date":1673276449}}'
        jsn_with_edited_msg = '{"update_id":319349776,"edited_message":{"message_id":309,"from":{"id":123,"is_bot":false,"first_name":"w","last_name":"w"},"chat":{"id":6666,"title":"title","type":"group"},"date":1673276449}}'
        jsns = [jsn_with_msg, jsn_with_edited_msg]
        for jsn in jsns:
            result = ts.get_chatid_from_str(jsn)
            self.assertEqual(result, 6666)
