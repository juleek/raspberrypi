import sys
from google.cloud import bigquery
import google.api_core.exceptions

dataset_id = "MainDataSet"

# https://cloud.google.com/functions/docs/bestpractices/tips#functions-graceful-termination-python
# https://googleapis.github.io/google-cloud-python/latest/bigquery/usage/datasets.html
def init():
   client = bigquery.Client();
   datasets = list(client.list_datasets())
   # print (datasets)
   has_required_dataset = bool(datasets and [True for dataset in datasets if dataset.dataset_id == dataset_id])
   # print (has_required_dataset);
   # print('Datasets in project "{}":'.format(project))
   # for dataset in datasets:  # API request(s)
   #     print('\t{}'.format(dataset.dataset_id))
   if has_required_dataset == False:
      print('"{}" project does not contain any datasets => creating "{}".'.format(client.project, dataset_id))
      dataset_ref = client.dataset(dataset_id)
      dataset = bigquery.Dataset(dataset_ref)
      dataset.location = 'europe-west2'
      try:
         dataset = client.create_dataset(dataset)  # API request
      except google.api_core.exceptions.AlreadyExists as exc:
         print('{}: {}'.format(type(exc).__name__, exc))
      except google.api_core.exceptions.Conflict as exc:
         print('{}: {}'.format(type(exc), exc))


global_var = init()

def hello_pubsub(data, context): # https://cloud.google.com/functions/docs/writing/background
    """Background Cloud Function to be triggered by Pub/Sub.
    Args:
         data (dict): The dictionary with data specific to this type of event.
         context (google.cloud.functions.Context): The Cloud Functions event
         metadata.
    """
    import base64

    if 'data' in data:
        name = base64.b64decode(data['data']).decode('utf-8')
    else:
        name = 'World'
    print ('Hello, {}!'.format(name))
