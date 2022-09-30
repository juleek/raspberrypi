#!/usr/bin/python3

from google.cloud import bigquery
import typing as t


class BigQueryDB():
    def __init__(self, project: str, dataset_id: str, location: str):
        self.client = bigquery.Client(project=project, location=location)
        self.dataset: bigquery.Dataset = self.create_dataset(dataset_id)

    def create_dataset(self, dataset_id: str) -> bigquery.Dataset:
        dataset_ref: bigquery.Dataset = bigquery.Dataset(f"{self.client.project}.{dataset_id}")
        dataset_ref.location = self.client.location
        return self.client.create_dataset(dataset_ref, exists_ok=True)

    def create_table(self,
                     dataset: bigquery.Dataset,
                     table_name: str,
                     fields: t.List[bigquery.SchemaField],
                     schema = None,
                     modify_table_callback: t.Optional[t.Callable[[bigquery.Table], bigquery.Table]] = lambda x: x) -> bigquery.Table:
        table_ref = dataset.table(table_name)
        table: bigquery.Table = bigquery.Table(table_ref, schema)
        table = modify_table_callback(table)

        self.client.create_table(table, exists_ok=True)

        original_schema: t.List[bigquery.SchemaField] = table.schema
        new_schema: t.List[bigquery.SchemaField] = original_schema[:]

        column_names_in_sc: t.List[str] = [field.name for field in original_schema]

        for field in fields:
            if field.name in column_names_in_sc:
                continue
            new_schema.append(field)

        table.schema = new_schema
        return self.client.update_table(table, ["schema"])
