use anyhow::{anyhow, Context, Result};

type MeasurementTx = tokio::sync::broadcast::Sender<common::Measurement>;

#[derive(Clone)]
pub struct Agg {
   tx: MeasurementTx,
   db: crate::db::measurement::Sqlite,
}

impl Agg {
   pub fn start(
      routes: tonic::service::Routes,
      db: crate::db::measurement::Sqlite,
   ) -> (tonic::service::Routes, MeasurementTx) {
      let (tx, _) = tokio::sync::broadcast::channel(16);
      let agg = Agg { tx: tx.clone(), db };
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
      let db = self.db.clone();

      use futures::StreamExt;
      let output = async_stream::try_stream! {
         loop {
            match stream.message().await {
               Ok(Some(proto)) => {
                  let response = match persist(proto, &tx, &db).await {
                     Ok(response) => response,
                     Err(why) => {
                        log::warn!("Failed to persist: {proto:?}: {why:?}");
                        continue;
                     }
                  };
                  yield response;

               }
               Ok(None) => {
                  log::warn!("Stream.message().await returned None");
                  continue;
               }
               Err(why) => {
                  log::warn!("Stream.message().await returned: {why:?}");
                  continue;
               }
            }
         }
      }
      .boxed();

      Ok(tonic::Response::new(output as PBStream))
   }
}


async fn persist(
   proto: common::pb::StoreMeasurementReq,
   tx: &MeasurementTx,
   db: &crate::db::measurement::Sqlite,
) -> Result<common::pb::StoreMeasurementResp> {
   log::info!("Received measurement: {:?}", proto);
   let measurement: common::Measurement = proto
      .measurement
      .clone()
      .ok_or_else(|| anyhow!("Measurement is missing in {proto:?}"))?
      .try_into()
      .with_context(|| anyhow!("Failed to convert proto measurement to measurement: {proto:?}"))?;

   use crate::db::measurement::Db;
   db.write(&measurement)
      .await
      .with_context(|| anyhow!("Failed to db.write {measurement:?}"))?;
   let confirmed = measurement.id.clone();
   let _ = tx.send(measurement);

   Ok(common::pb::StoreMeasurementResp {
      confirmed: Some(confirmed.into()),
   })
}
