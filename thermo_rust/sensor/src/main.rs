use anyhow::Result;


#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
   /// For example http://127.0.0.1:12345
   #[arg(long)]
   server_host_port: String,

   /// Path to bottom sensor file
   #[arg(long)]
   bottom: std::path::PathBuf,

   /// Path to ambient sensor file
   #[arg(long)]
   ambient: std::path::PathBuf,

   // Logger's level
   #[arg(long)]
   #[arg(value_parser = clap::builder::PossibleValuesParser::new(["error", "warn", "info", "debug", "trace"]))]
   #[arg(default_value_t = String::from("info"))]
   log_level: String,
}

// TODO: make sure that all interesting parts are logged:
// * Size of vector with measurements in publish (for example if its len() is > 100)
// * Size of channel if it len() is > 10.


#[tokio::main]
async fn main() -> Result<()> {
   use clap::Parser;
   let cli = Cli::parse();
   common::init_logger(&cli.log_level);

   let ct = tokio_util::sync::CancellationToken::new();

   {
      let ct = ct.clone();
      ctrlc::set_handler(move || ct.cancel()).unwrap();
   }

   let rx = sensor::sensor::spawn_pollers(&cli.bottom, &cli.ambient, &ct);

   sensor::publisher::poll_and_publish_forever(&ct, rx, &cli.server_host_port).await?;
   Ok(())
}
