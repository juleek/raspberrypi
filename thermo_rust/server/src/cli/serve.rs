use anyhow::{anyhow, Context, Result};


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
}

impl Cli {
   pub async fn run(&self) -> Result<()> {
      let routes = tonic::service::Routes::default();
      let pool = crate::db::Location::Path(self.db_path.clone()).create_pool().await?;
      let sqlite = crate::db::measurement::Sqlite::new(&pool).await?;
      let (routes, _tx) = crate::grpc::Agg::start(routes, sqlite);
      // let sender = std::sync::Arc::new(crate::message::Telegram { chat_id: 123456789,
      //                                                              bot_id:  "wwwwwww".to_string(), });
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
