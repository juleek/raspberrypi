use anyhow::{Context, Result, anyhow};


/// The main mode where we start server that listens for incoming measurements
#[derive(clap::Parser, Debug)]
pub struct Cli {
   /// Host and port to listen on server
   #[arg(long)]
   host_port: String,

   #[arg(long)]
   db_path: std::path::PathBuf,

   #[command(flatten)]
   tls: common::tls::ServerArgs,

   #[command(flatten)]
   telegram: crate::message::TelegramArgs,
}

impl Cli {
   pub async fn run(&self) -> Result<()> {
      let routes = tonic::service::Routes::default();
      let pool = crate::db::Location::Path(self.db_path.clone()).create_pool().await?;
      let measuruments_db = crate::db::measurement::Sqlite::new(&pool).await?;
      let sensor_db = crate::sensor::Sqlite::new(&pool).await?;

      let (routes, _tx) = crate::grpc::Agg::start(routes, measuruments_db.clone());
      let sender = crate::message::Telegram::from_args(self.telegram.clone());
      crate::cron::start(&measuruments_db, &sensor_db, sender)
         .with_context(|| anyhow!("Failed to start cron"))?;

      let addr: std::net::SocketAddr =
         self.host_port.parse().with_context(|| anyhow!("Failed to parse: {}", self.host_port))?;
      tonic::transport::Server::builder()
         .tls_config(self.tls.server_tls_config()?)?
         .add_routes(routes)
         .serve(addr)
         .await?;
      Ok(())
   }
}
