use anyhow::{anyhow, Context, Result};

use server::alerting;
use server::sender;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
   #[arg(long)]
   min_temp_bottom:  f64,
   #[arg(long)]
   min_temp_ambient: f64,
}


struct Agg {
   counter: std::sync::Mutex<i32>,
   tx:      tokio::sync::broadcast::Sender<agg_proto::MeasurementReq>,
}


type Stream = dyn futures::Stream<Item = Result<agg_proto::MeasurementResp, tonic::Status>> + Send;
type PBStream = std::pin::Pin<Box<Stream>>;


#[tonic::async_trait]
impl agg_proto::agg_server::Agg for Agg {
   type SendMeasurementStream =
      std::pin::Pin<Box<dyn futures::Stream<Item = Result<agg_proto::MeasurementResp, tonic::Status>> + Send>>;

   async fn send_measurement(&self,
                             request: tonic::Request<tonic::Streaming<agg_proto::MeasurementReq>>)
                             -> Result<tonic::Response<Self::SendMeasurementStream>, tonic::Status> {
      let mut stream = request.into_inner();
      let tx = self.tx.clone();

      use futures::StreamExt;
      let output = async_stream::try_stream! {
                       while let Some(measurements_with_counter) = stream.message().await.unwrap_or(None) {
                           println!("Server received: {:?}", measurements_with_counter);

                           //DB

                           tx.send(measurements_with_counter.clone()).with_context(|| anyhow!("Failed to send measurement"));

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




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   helpers::helpers::init_logger("debug");
   use clap::Parser;
   let cli = Cli::parse();

   let (tx, _) = tokio::sync::broadcast::channel(10);

   let sender: std::sync::Arc<Box<dyn crate::sender::Sender>> =
      Arc::new(Box::new(crate::sender::TelegramSender { chat_id: 123456789,
                                                        bot_id:  "wwwwwww".to_string(), }));

   tokio::spawn(alerting::send_alert_message_if_needed(tx.subscribe(),
                                                    &cli.min_temp_bottom,
                                                    &cli.min_temp_ambient,
                                                    sender.clone()));

   let agg = Agg { counter: std::sync::Mutex::new(10),
                   tx:      tx.clone(), };
   let agg = agg_proto::agg_server::AggServer::new(agg);



   tonic::transport::Server::builder().add_service(agg)
                                      .serve("0.0.0.0:12345".parse().unwrap())
                                      .await?;
   Ok(())
}
