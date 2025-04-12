use anyhow::Result;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
   /// Threshold for bottom temperature
   #[arg(long)]
   min_temp_bottom: f64,

   /// Threshold for ambient temperature
   #[arg(long)]
   min_temp_ambient: f64,

   /// Port to listen on server
   #[arg(long)]
   host_port: String,
   // http port for webhook
}





#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   common::init_logger("debug");
   use clap::Parser;
   let cli = Cli::parse();
   let routes = tonic::service::Routes::default();
   let (routes, tx) = server::grpc::Agg::start(routes);
   // let sender = std::sync::Arc::new(server::message::Telegram { chat_id: 123456789,
   //                                                              bot_id:  "wwwwwww".to_string(), });
   // DB
   // server::alerting::start();
   // server::notifier::start();
   // server::webhook::start();
   tonic::transport::Server::builder().add_routes(routes)
                                      .serve(cli.host_port.parse().unwrap())
                                      .await?;
   Ok(())
}
