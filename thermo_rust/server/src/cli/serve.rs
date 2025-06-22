use anyhow::Result;


/// The main mode where we start server that listens for incoming measurements
#[derive(clap::Parser, Debug)]
pub struct Cli {
   /// Port to listen on server
   #[arg(long)]
   host_port: String,

   #[arg(long)]
   db_path: String,
}

impl Cli {
   pub async fn run(&self) -> Result<()> {
      // let routes = tonic::service::Routes::default();
      // let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      // let sqlite = crate::db::measurement::Sqlite::new(&pool).await?;
      todo!();
      // let (routes, tx) = crate::grpc::Agg::start(routes, sqlite);
      // // let sender = std::sync::Arc::new(crate::message::Telegram { chat_id: 123456789,
      // //                                                              bot_id:  "wwwwwww".to_string(), });
      // // DB
      // // crate::alerting::start();
      // // crate::notifier::start();
      // // crate::webhook::start();
      // tonic::transport::Server::builder().add_routes(routes)
      //                                    .serve(self.host_port.parse().unwrap())
      //                                    .await?;
      // Ok(())
   }
}
