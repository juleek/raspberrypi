from pathlib import Path
import sys
from google.cloud import bigquery
import google.api_core.exceptions
from typing import List, Tuple


class GBigQuery:
    def __init__(self, dataset_id: str, location: str, dry_run: bool) -> None:
        """
        Creates dataset, table in location in BigQuery
        """
        self.dataset_id = dataset_id
        self.location = location
        self.dry_run = dry_run

        self.client = bigquery.Client()

        # https://cloud.google.com/functions/docs/bestpractices/tips#functions-graceful-termination-python
        # https://googleapis.github.io/google-cloud-python/latest/bigquery/usage/datasets.html

        # datasets = list(self.client.list_datasets())
        # has_required_dataset = bool(datasets and
        #                             [True for dataset in datasets if dataset.dataset_id == self.dataset_id])

        if self.dry_run:
            return
        self.dataset_ref = self.client.dataset(self.dataset_id)
        try:
            self.client.get_dataset(self.dataset_ref)
        except google.api_core.exceptions.NotFound:
            print('Project "{}" does not contain dataset "{}" => creating it'.format(self.client.project,
                                                                                     self.dataset_id))
            self.__create_dataset()

    @classmethod
    def dry_run(cls):
        result = cls(dataset_id="", location="", dry_run=True)
        return result

    @classmethod
    def wet_run(cls, dataset_id: str, location: str):
        result = cls(dataset_id=dataset_id, location=location, dry_run=False)
        return result

    def __create_dataset(self) -> None:
        dataset = bigquery.Dataset(self.dataset_ref)
        dataset.location = self.location
        try:
            dataset = self.client.create_dataset(dataset)  # API request
            print('Dataset "{}" created.\n'.format(dataset.dataset_id))
        except google.api_core.exceptions.AlreadyExists as exc:
            print('{}: {}'.format(type(exc).__name__, exc))
        except google.api_core.exceptions.Conflict as exc:
            print('{}: {}'.format(type(exc), exc))

    def get_table_or_none(self, table_id: str):
        table_ref = self.dataset_ref.table(table_id)
        try:
            table = self.client.get_table(table_ref)
        except google.api_core.exceptions.NotFound:
            return None
        return table

    def create_table_if_not_created(self, table_id: str):
        table = self.get_table_or_none(table_id=table_id)
        if not table:
            print('Dataset "{}" in project "{}" does not contain table "{}" => creating it'.
                  format(self.dataset_id, self.client.project, table_id))
            table = self.create_table(table_id=table_id)
        return table

    def create_table(self, table_id: str):
        table_ref = self.dataset_ref.table(table_id)
        try:
            self.client.create_table(bigquery.Table(table_ref))  # API request
            print('Table "{}" created.\n'.format(table_ref))
        except google.api_core.exceptions.Conflict as exc:
            print('Got exception while creating table {}: {}'.format(type(exc), exc))

        table = self.client.get_table(table_ref)
        return table

    def delete_table(self, table_id: str):
        table_ref = self.dataset_ref.table(table_id)
        self.client.delete_table(table_ref)
        print('Table {}:{} deleted.'.format(self.dataset_id, table_id))

    def ensure_table_scheme(self, existing_table, schema: List[bigquery.SchemaField]):
        if self.dry_run:
            return

        original_schema = existing_table.schema
        # print ('Original schema: {}'.format(original_schema))
        new_schema = original_schema[:]  # creates a copy of the schema
        for schema_field in schema:
            if not [True for existing_field in original_schema if existing_field.name == schema_field.name]:
                print('{} is not in original schema of table "{}" => adding it'.format(schema_field,
                                                                                       existing_table.table_id))
                new_schema.append(schema_field)

        existing_table.schema = new_schema
        table = self.client.update_table(existing_table, ['schema'])  # API request
        return table

    def insert_rows(self, table, rows_to_insert: List[Tuple]) -> None:
        """
        Inserts its arguments in BigQuery
        https://cloud.google.com/bigquery/streaming-data-into-bigquery#bigquery-stream-data-python
        """

        print('Inserting: {} into table: {}'.format(rows_to_insert, table.table_id))

        if self.dry_run:
            return

        errors = self.client.insert_rows(table, rows_to_insert)  # API request
        if errors:
            print(errors)
            assert errors == []
