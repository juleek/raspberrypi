use anyhow::Result;


#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
   /// For example http://127.0.0.1:12345
   #[arg(long)]
   server_host_port: String,

   /// Bottom sensor id
   #[arg(long)]
   bottom_id: String,

   /// Path to bottom sensor file
   #[arg(long)]
   bottom_path: std::path::PathBuf,

   /// Ambient sensor id
   #[arg(long)]
   ambient_id: String,

   /// Path to ambient sensor file
   #[arg(long)]
   ambient_path: std::path::PathBuf,

   /// How often to poll sensors, in seconds
   #[arg(long, default_value_t = 20)]
   sensor_poll_periodicity: i32,

   /// Id of location (short random string)
   #[arg(long)]
   location: String,

   // Logger's level
   #[arg(long)]
   #[arg(value_parser = clap::builder::PossibleValuesParser::new(["error", "warn", "info", "debug", "trace"]))]
   #[arg(default_value_t = String::from("info"))]
   log_level: String,
}
impl Cli {
   fn sensor_poll_periodicity(&self) -> std::time::Duration {
      std::time::Duration::from_secs(self.sensor_poll_periodicity as u64)
   }
}


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

   let sensor_metas = &[
      sensor::sensor::Meta {
         id: cli.bottom_id.clone().try_into()?,
         path: cli.bottom_path.clone(),
      },
      sensor::sensor::Meta {
         id: cli.ambient_id.clone().try_into()?,
         path: cli.ambient_path.clone(),
      },
   ];

   let rx = sensor::sensor::spawn_pollers(sensor_metas, cli.sensor_poll_periodicity(), &ct);

   sensor::publisher::poll_and_publish_forever(&ct, rx, &cli.server_host_port).await?;
   Ok(())
}
