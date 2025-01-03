use anyhow::{anyhow, Context, Result};
use sensor::publisher;


#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
   /// For example http://127.0.0.1:12345
   #[arg(long)]
   server_host_port: String,
   bottom:           String,
   ambient:          String,
}

// pub-sub: publisher & subscriber


fn init_logger(log_level: &str) {
   let mut builder = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level));
   builder.format_timestamp_micros();
   builder.init();
}



fn main() -> Result<()> {
   use clap::Parser;
   let cli = Cli::parse();
   init_logger("debug");

   let ct = tokio_util::sync::CancellationToken::new();

   {
      let ct = ct.clone();
      ctrlc::set_handler(move || ct.cancel()).map_err(anyhow::Error::new)?;
   }

   let (tx, rx) = tokio::sync::mpsc::channel(10);
   let rt = sensor::sensor::create_tasks(&tx, &cli.bottom, &cli.ambient, &ct);

   rt.block_on(publisher::poll_and_publish_forever(&ct, rx, &cli.server_host_port))?;
   Ok(())
}
