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
         while let Some(measurements_with_counter) = stream.message().await.unwrap_or(None) {
            println!("Server received: {:?}", measurements_with_counter);

            //DB

            tx.send(measurements_with_counter.measurement.unwrap().into()).with_context(|| anyhow!("Failed to send measurement"));

            let response = common::pb::MeasurementResp {
                  counter: measurements_with_counter.counter,
            };

            println!("Server sending: {:?}", response);
            yield response;
         }
      }
      .boxed();

      Ok(tonic::Response::new(output as PBStream))
   }
}
