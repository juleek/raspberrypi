use anyhow::Result;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
   /// Log-level. We are using env_logger, so everything can be configured with env-vars, but we also
   /// provide an option to configure it with a command-line arguments.
   /// See more info at https://docs.rs/env_logger/0.10.0/env_logger/
   // #[arg(value_enum, long, default_value_t = LogLevel::default())]
   #[arg(long)]
   #[arg(value_parser = clap::builder::PossibleValuesParser::new(["error", "warn", "info", "debug", "trace"]))]
   #[arg(default_value_t = String::from("info"))]
   log_level: String,

   #[command(subcommand)]
   command: server::cli::Commands,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   use clap::Parser;
   let cli = Cli::parse();
   common::init_logger(&cli.log_level);

   cli.command.run().await?;
   Ok(())
}
