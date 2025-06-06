use anyhow::{anyhow, Context, Result};

type MeasurementTx = tokio::sync::broadcast::Sender<common::Measurement>;

#[derive(Clone)]
pub struct Agg {
   tx: MeasurementTx,
}

impl Agg {
   pub fn start(routes: tonic::service::Routes) -> (tonic::service::Routes, MeasurementTx) {
      let (tx, _) = tokio::sync::broadcast::channel(16);
      let agg = Agg { tx: tx.clone() };
      let service = common::pb::aggproto::agg_server::AggServer::new(agg);
      let routes = routes.add_service(service);
      (routes, tx)
   }
}


// ===========================================================================================================
// GRPC service

type Stream = dyn futures::Stream<Item = Result<common::pb::StoreMeasurementResp, tonic::Status>> + Send;
type PBStream = std::pin::Pin<Box<Stream>>;

#[tonic::async_trait]
impl common::pb::agg_server::Agg for Agg {
   type StoreMeasurementStream = std::pin::Pin<
      Box<dyn futures::Stream<Item = Result<common::pb::StoreMeasurementResp, tonic::Status>> + Send>,
   >;

   async fn store_measurement(
      &self,
      request: tonic::Request<tonic::Streaming<common::pb::StoreMeasurementReq>>,
   ) -> Result<tonic::Response<Self::StoreMeasurementStream>, tonic::Status> {
      let mut stream = request.into_inner();
      let tx = self.tx.clone();

      use futures::StreamExt;
      let output = async_stream::try_stream! {
         while let Some(proto) = stream.message().await.unwrap_or(None) {
            let response = on_measurement(proto, &tx);
            let response = match response {
               Ok(response) => response,
               Err(why) => {
                  println!("Failed to send measurement: {why:?}");
                  continue;
               }
            };
            yield response;
         }
      }
      .boxed();

      Ok(tonic::Response::new(output as PBStream))
   }
}


fn on_measurement(
   proto: common::pb::StoreMeasurementReq,
   tx: &MeasurementTx,
) -> Result<common::pb::StoreMeasurementResp> {
   println!("Server received: {:?}", proto);
   let measurement: common::Measurement = proto
      .measurement
      .clone()
      .ok_or_else(|| anyhow!("Measurement is missing from {proto:?}"))?
      .try_into()?;
   if measurement.id.location.is_empty() || measurement.id.sensor.is_empty() {
      return Err(anyhow!("One of Measurement fields is empty: {measurement:?}"));
   }
   //DB
   let confirmed = measurement.clone().id;
   let _ = tx.send(measurement);

   Ok(common::pb::StoreMeasurementResp {
      confirmed: Some(confirmed.into()), // fixed 4.06.25
   })
}
