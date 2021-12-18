### Set Webhook:
```
curl -v "https://api.telegram.org/bot____/setWebhook?url=____"
```


### Google functions deploy

```
Monitoring bot:
gcloud functions deploy ___ --entry-point=_____ --runtime=python39 --trigger-http --region=europe-west1 --source=_____


Alerting Bot:
gcloud functions deploy ____ --entry-point=____ --runtime=python39 --trigger-http --region=europe-west1 --source=___


Telemetry Bot:
gcloud functions deploy ____ --entry-point=____ --runtime=python39 --trigger-topic=topic-for-telemetry --region=europe-west1 --source=____
```
