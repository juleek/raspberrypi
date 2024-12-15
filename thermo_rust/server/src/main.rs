use anyhow::{anyhow, Context, Result};



struct Agg {
   counter: std::sync::Mutex<i32>,
}


type Stream =
   dyn futures::Stream<Item = Result<agg_proto::MeasurementResp, tonic::Status>> + Send;
type PBStream = std::pin::Pin<Box<Stream>>;


#[tonic::async_trait]
impl agg_proto::agg_server::Agg for Agg {
   type SendMeasurementStream =
      std::pin::Pin<Box<dyn futures::Stream<Item = Result<agg_proto::MeasurementResp, tonic::Status>> + Send>>;

   async fn send_measurement(&self,
                             request: tonic::Request<tonic::Streaming<agg_proto::MeasurementReq>>)
                             -> Result<tonic::Response<Self::SendMeasurementStream>, tonic::Status> {
      let mut stream = request.into_inner();

      use futures::StreamExt;
      let output = async_stream::try_stream! {
                       while let Some(measurements_with_counter) = stream.message().await.unwrap_or(None) {
                           println!("Server received: {:?}", measurements_with_counter);
                           let response = agg_proto::MeasurementResp {
                               counter: measurements_with_counter.counter,
                           };
                           println!("Server sending: {:?}", response);
                           yield response;
                       }
                   }.boxed();

      // Return the response stream as gRPC response
      Ok(tonic::Response::new(output as PBStream))
   }
}

fn init_logger(log_level: &str) {
   let mut builder = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level));
   builder.format_timestamp_micros();
   builder.init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   init_logger("debug");
   let stop_requested = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

   {
      let stop_requested = stop_requested.clone();
      ctrlc::set_handler(move || {
         stop_requested.store(true, std::sync::atomic::Ordering::Relaxed);
      }).map_err(anyhow::Error::new)?;
   }
   // let rt = tokio::runtime::Builder::new_multi_thread().enable_all()
   //                                                     .worker_threads(3)
   //                                                     .thread_name("tokio")
   //                                                     .build()
   //                                                     .unwrap();

   let agg = Agg { counter: std::sync::Mutex::new(10), };
   let agg = agg_proto::agg_server::AggServer::new(agg);

   tonic::transport::Server::builder().add_service(agg)
                                      .serve("0.0.0.0:12345".parse().unwrap())
                                      .await?;
   Ok(())
}
