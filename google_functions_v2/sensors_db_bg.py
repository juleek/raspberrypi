from google.cloud import bigquery
import google
import sensor_db as sdb
from devicedatum import DeviceDatum
import datetime as dt
import typing as t
import sensor as sen
import bisect as bs
from collections import defaultdict
import pytz

class SensorsDBBQ(sdb.SensorsDB):
    def __init__(self, project: str, dataset_id: str, table_name: str, location: str):
        self.client = bigquery.Client(project=project, location=location)
        self.dataset: bigquery.Dataset = self.create_dataset(dataset_id)
        self.table = self.create_table(self.dataset, table_name)
        self.ensure_fields_are_in_schema(self.table, [
            bigquery.SchemaField("ErrorMessage", "STRING", mode="NULLABLE"),
            bigquery.SchemaField("Tubes",
                                 "RECORD",
                                 mode="REPEATED",
                                 fields=[
                                     bigquery.SchemaField("TubeName", "STRING", mode="REQUIRED"),
                                     bigquery.SchemaField("Temperature", "FLOAT", mode="REQUIRED")])
        ])


    def create_dataset(self, dataset_id: str) -> bigquery.Dataset:
        dataset_ref: bigquery.Dataset = bigquery.Dataset(f"{self.client.project}.{dataset_id}")
        dataset_ref.location = self.client.location
        return self.client.create_dataset(dataset_ref, exists_ok=True)


    def create_table(self, dataset: bigquery.Dataset, table_name: str) -> bigquery.Table:
        table_ref = dataset.table(table_name)
        schema = [bigquery.SchemaField("Timestamp", "TIMESTAMP", mode="REQUIRED")]
        table: bigquery.Table = bigquery.Table(table_ref, schema)
        table.clustering_fields = ["Timestamp"]

        table.time_partitioning = bigquery.TimePartitioning(
            type_=bigquery.TimePartitioningType.MONTH,
            field="Timestamp",
            expiration_ms= int((365*3 + 365/2) * 24 * 60 * 60 * 1000)
        )

        return self.client.create_table(table, exists_ok=True)


    def ensure_fields_are_in_schema(self, table: bigquery.Table, fields: t.List[bigquery.SchemaField]) -> bigquery.Table:
        original_schema = table.schema
        new_schema = original_schema[:]

        column_names_in_sc: t.List = []
        for i in range(0, len(original_schema)):
            column_names_in_sc.append(original_schema[i].name)

        for field in fields:
            if field.name in column_names_in_sc:
                continue
            new_schema.append(field)

        table.schema = new_schema
        return self.client.update_table(table, ["schema"])



    def write(self, datum: DeviceDatum) -> None:
        tubes = []
        for name, temp in datum.name_to_temp.items():
            tubes.append({"TubeName": name, "Temperature": temp})

        str_datetime = datum.time.strftime("%Y-%m-%d %H:%M:%S.%f")
        if datum.error_msg == "":
            self.client.insert_rows(self.table, [{"Timestamp": str_datetime, "Tubes": tubes}])
        else:
            self.client.insert_rows(self.table, [{"Timestamp": str_datetime, "ErrorMessage": datum.error_msg, "Tubes": tubes}])



    def read_starting_from(self, date: dt.datetime) -> t.Tuple[t.List[sen.Sensor], t.Set[str]]:
        str_datetime = date.strftime("%Y-%m-%d %H:%M:%S.%f")
        query = f"SELECT * FROM {self.table} WHERE Timestamp >= TIMESTAMP('{str_datetime}') ORDER BY Timestamp"
        query_job = self.client.query(query)
        name_to_sensor_temp: t.Dict[str, sen.Sensor] = defaultdict(lambda: sen.Sensor(temperatures=[], name="", timestamps=[]))
        messages: t.Set[str] = set()

        for row in query_job:
            for item in row[2]:
                name_to_sensor_temp[item["TubeName"]].name = item["TubeName"]
                name_to_sensor_temp[item["TubeName"]].timestamps.append(row[0])
                name_to_sensor_temp[item["TubeName"]].temperatures.append(item["Temperature"])
            if row[1]:
                messages.add(row[1])
        sensors: t.List[sen.Sensor] = list(name_to_sensor_temp.values())
        return sensors, messages



    def delete_before(self, date: dt.datetime) -> None:
        str_datetime = date.strftime("%Y-%m-%d %H:%M:%S.%f")
        query = f"DELETE FROM {self.table}  WHERE Timestamp < TIMESTAMP('{str_datetime}')"
        self.client.query(query)



def main() -> None:
    pass
    # sensord_db: SensorsDBBQ = SensorsDBBQ(project="tarasovka", dataset_id="test", table_name="test_db", location="europe-west2")
    # datum: DeviceDatum = DeviceDatum({"self.tube": 25.1}, dt.datetime(2011, 11, 4, 0, 0, tzinfo=pytz.UTC), "")
    # sensord_db.write(datum)
    # sensord_db.read_starting_from(dt.datetime.now())



main()
