use anyhow::{anyhow, Context, Result};
use common::pb;

type MeasurementTx = tokio::sync::broadcast::Sender<common::Measurement>;

#[derive(Clone)]
pub struct Agg {
   tx: MeasurementTx,
}

impl Agg {
   pub fn start(routes: tonic::service::Routes) -> (tonic::service::Routes, MeasurementTx) {
      let (tx, _) = tokio::sync::broadcast::channel(16);
      let agg = Agg { tx: tx.clone() };
      let service = pb::aggproto::agg_server::AggServer::new(agg);
      let routes = routes.add_service(service);
      (routes, tx)
   }
}


// ===========================================================================================================
// GRPC service

type Stream = dyn futures::Stream<Item = Result<common::pb::MeasurementResp, tonic::Status>> + Send;
type PBStream = std::pin::Pin<Box<Stream>>;

#[tonic::async_trait]
impl common::pb::agg_server::Agg for Agg {
   type SendMeasurementStream = std::pin::Pin<
      Box<dyn futures::Stream<Item = Result<common::pb::MeasurementResp, tonic::Status>> + Send>,
   >;

   async fn send_measurement(
      &self,
      request: tonic::Request<tonic::Streaming<common::pb::MeasurementReq>>,
   ) -> Result<tonic::Response<Self::SendMeasurementStream>, tonic::Status> {
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
   proto: common::pb::MeasurementReq,
   tx: &MeasurementTx,
) -> Result<common::pb::MeasurementResp> {
   println!("Server received: {:?}", proto);
   let measurement = proto
      .measurement
      .clone()
      .ok_or_else(|| anyhow!("Measurement is missing from {proto:?}"))?
      .try_into()?;
   //DB
   let _ = tx.send(measurement);
   Ok(common::pb::MeasurementResp {
      counter: proto.counter,
   })
}
