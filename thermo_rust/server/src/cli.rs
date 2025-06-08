use anyhow::Result;

pub mod config;
pub mod serve;


#[derive(clap::Subcommand, Debug)]
pub enum Commands {
   Serve(serve::Cli),
   Config(config::Cli),
}


impl Commands {
   pub async fn run(&self) -> Result<()> {
      match self {
         Commands::Serve(cli) => cli.run().await,
         Commands::Config(cli) => cli.run().await,
      }
   }
}
