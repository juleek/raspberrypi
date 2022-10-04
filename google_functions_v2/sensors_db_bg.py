from google.cloud import bigquery
import sensor_db as sdb
import datetime as dt
import typing as t
import sensor as sen
from collections import defaultdict
import bigquerydb as bigdb
import devicedatum as dd

class SensorsDBBQ(sdb.SensorsDB):
    COL_TIMESTAMP_NAME: str = "Timestamp"
    COL_ERROR_MSG_NAME: str = "ErrorMessage"
    COL_TUBENAME_NAME: str = "TubeName"
    COL_TEMPERATURE_NAME: str = "Temperature"
    COL_TUBES_NAME: str = "Tubes"
    TABLE_NAME: str = "sensors_data_table"
    TIME_FORMAT: str = "%Y-%m-%d %H:%M:%S.%f"

    def __init__(self, project: str, dataset_id: str, location: str):
        self.db = bigdb.BigQueryDB(project=project, dataset_id=dataset_id, location=location)
        self.table = self.create_sensors_table()


    def create_sensors_table(self) -> bigquery.Table:

        def set_up_partitioning(tbl: bigquery.Table) -> bigquery.Table:
            tbl.clustering_fields = [self.COL_TIMESTAMP_NAME]

            tbl.time_partitioning = bigquery.TimePartitioning(
                type_=bigquery.TimePartitioningType.MONTH,
                field=self.COL_TIMESTAMP_NAME,
                expiration_ms=int((365*3 + 365/2) * 24 * 60 * 60 * 1000))
            return tbl

        table = self.db.create_table(table_name=self.TABLE_NAME,
                                     schema=[bigquery.SchemaField(self.COL_TIMESTAMP_NAME, "TIMESTAMP", mode="REQUIRED")],
                                     fields=[bigquery.SchemaField(self.COL_ERROR_MSG_NAME, "STRING", mode="NULLABLE"),
                                             bigquery.SchemaField(self.COL_TUBES_NAME, "RECORD", mode="REPEATED",
                                                                  fields=[bigquery.SchemaField(self.COL_TUBENAME_NAME, "STRING", mode="REQUIRED"),
                                                                          bigquery.SchemaField(self.COL_TEMPERATURE_NAME, "FLOAT", mode="REQUIRED")])],
                                     modify_table_callback=set_up_partitioning)

        return table


    def write(self, datum: dd.DeviceDatum) -> None:
        tubes: t.List[t.Dict[str, float]] = []
        for name, temp in datum.name_to_temp.items():
            tubes.append({self.COL_TUBENAME_NAME: name, self.COL_TEMPERATURE_NAME: temp})

        str_datetime: str = datum.time.strftime(self.TIME_FORMAT)
        if datum.error_msg == "":
            self.db.client.insert_rows(self.table, [{self.COL_TIMESTAMP_NAME: str_datetime, self.COL_TUBES_NAME: tubes}])
        else:
            self.db.client.insert_rows(self.table,
                                    [{self.COL_TIMESTAMP_NAME: str_datetime,
                                      self.COL_ERROR_MSG_NAME: datum.error_msg,
                                      self.COL_TUBES_NAME: tubes}])



    def read_starting_from(self, date: dt.datetime) -> t.Tuple[t.List[sen.Sensor], t.Set[str]]:
        str_datetime: str = date.strftime(self.TIME_FORMAT)
        query = f"SELECT {self.COL_TIMESTAMP_NAME}, {self.COL_ERROR_MSG_NAME}, {self.COL_TUBES_NAME} FROM {self.table} WHERE {self.COL_TIMESTAMP_NAME} >= TIMESTAMP('{str_datetime}') ORDER BY {self.COL_TIMESTAMP_NAME}"
        query_job = self.db.client.query(query)
        name_to_sensor_temp: t.Dict[str, sen.Sensor] = defaultdict(lambda: sen.Sensor(temperatures=[], name="", timestamps=[]))
        messages: t.Set[str] = set()

        for row in query_job:
            for item in row[2]:
                name_to_sensor_temp[item[self.COL_TUBENAME_NAME]].name = item[self.COL_TUBENAME_NAME]
                name_to_sensor_temp[item[self.COL_TUBENAME_NAME]].timestamps.append(row[0])
                name_to_sensor_temp[item[self.COL_TUBENAME_NAME]].temperatures.append(item[self.COL_TEMPERATURE_NAME])
            if row[1]:
                messages.add(row[1])
        return list(name_to_sensor_temp.values()), messages



    def delete_before(self, date: dt.datetime) -> None:
        str_datetime: str = date.strftime(self.TIME_FORMAT)
        query = f"DELETE FROM {self.table}  WHERE {self.COL_TIMESTAMP_NAME} < TIMESTAMP('{str_datetime}')"
        self.db.client.query(query)



# if __name__ == "__main__":
#     db = SensorsDBBQ(project="tarasovka", dataset_id="tarasovka", location="europe-west2")
#     datum_1: dd.DeviceDatum = dd.DeviceDatum({"Ambient": 11.2, "BottomTube": 15.6}, dt.datetime(2022, 9, 29, 2, 0, tzinfo=pytz.UTC), "")
#     datum_2: dd.DeviceDatum = dd.DeviceDatum({"Ambient": 13.2, "BottomTube": 18.6}, dt.datetime(2022, 9, 29, 2, 2, tzinfo=pytz.UTC), "")
#     db.write(datum_1)
#     db.write(datum_2)
