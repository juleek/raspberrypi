from google.cloud import pubsub_v1
from google.api_core import exceptions
from logger import logger


def create_topic_if_not_exists(client: pubsub_v1.PublisherClient, topic_path: str):
    try:
        client.get_topic(topic=topic_path)
    except exceptions.NotFound:
        client.create_topic(request={"name": topic_path})


def create_topic_and_publish_msg(project_id: str, topic_id: str, msg: str):
    publisher: pubsub_v1.PublisherClient = pubsub_v1.PublisherClient()
    topic_path: str = publisher.topic_path(project_id, topic_id)
    create_topic_if_not_exists(publisher, topic_path)

    future = publisher.publish(topic_path, msg.encode())
    message_id = future.result()
    logger.info(f'message_id = {message_id}')
