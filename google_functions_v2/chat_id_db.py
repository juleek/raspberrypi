#!/usr/bin/python3

from google.cloud import bigquery
import bigquerydb as bigdb
import sender as sen
import typing as t

class ChatIdDB:
    COL_BOT_NAME: str = "bot_name"
    COL_CHAT_ID: str = "chat_id"
    TABLE_NAME: str = "chat_ids"

    def __init__(self, db: bigdb.BigQueryDB):
        self.db = db
        self.table = self.db.create_table(table_name=self.TABLE_NAME,
                                          fields=[bigquery.SchemaField(self.COL_BOT_NAME, "STRING", mode="REQUIRED"),
                                                  bigquery.SchemaField(self.COL_CHAT_ID, "INTEGER", mode="REQUIRED")])


    def ask_to_add(self, chat_id: int, sender: sen.Sender, bot_name: str):
        text: str = f'Authenticating has not been implemented yet, so insert your chat id into Google BigQuery manually by issuing:\n\n' \
                    f'```SQL\n' \
                    f'MERGE INTO {self.table} AS Dst\n' \
                    f'USING (SELECT "{bot_name}" AS Bot_name, {chat_id} AS Chat_ID) AS Src\n' \
                    f'ON Dst.Bot_name = Src.Bot_name\n' \
                    f'WHEN MATCHED THEN UPDATE SET Chat_ID = Src.Chat_ID\n' \
                    f'WHEN NOT MATCHED THEN INSERT (Bot_name, Chat_ID) VALUES (Src.Bot_name, Src.Chat_ID);\n' \
                    f'```\n' \
                    f'at https://console\\.cloudgoogle\\.com/bigquery'

        sender.send_text(text, is_markdown=True)


    def read(self, bot_name: str) -> t.Optional[int]:
        query: str = f"SELECT {self.COL_CHAT_ID} FROM {self.table} WHERE {self.COL_BOT_NAME} = '{bot_name}'"
        query_job: bigquery.job.QueryJob = self.db.client.query(query)

        for row in query_job:
            return row[0]
        return None


# if __name__ == "__main__":
#     import telegram_sender as tels
#     import secrets_bot
#     BOT_SECRET: str = secrets_bot.notifier_bot_id
#     db_bigq = bigdb.BigQueryDB(project="tarasovka", dataset_id="tarasovka", location="europe-west2")
#     db = ChatIdDB(db_bigq)
#     db.ask_to_add(chat_id=-748244195, sender=tels.TelegramSender(-748244195, BOT_SECRET), bot_name="notifier_bot")
