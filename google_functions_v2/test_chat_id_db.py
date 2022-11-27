import unittest
import chat_id_db as chiddb
import stdout_sender as stdout
import bigquerydb as bigqr

@unittest.skip("This is a manual test")
class TestChatIdDB(unittest.TestCase):
    def setUp(self):
        self.db: chiddb.ChatIdDB = chiddb.ChatIdDB(bigqr.BigQueryDB(project="tarasovka", dataset_id="test", location="europe-west2"))
        self.chat_id: int = 12345656565
        self.bot_name: str = "Notifier_bot"

    def test_ask_to_add(self):
        self.db.ask_to_add(self.chat_id, stdout.StdoutSender(), self.bot_name)

    def test_read_chat_id_from_db(self):
        result = self.db.read(self.bot_name)
        self.assertEqual(result, self.chat_id)
