use anyhow::Result;

pub mod config;
pub mod serve;
pub mod tls;


#[derive(clap::Subcommand, Debug)]
pub enum Workflow {
   Serve(serve::Cli),
   Config(config::Cli),
   Tls(tls::Cli),
}

impl Workflow {
   pub async fn run(&self) -> Result<()> {
      match self {
         Workflow::Serve(cli) => cli.run().await,
         Workflow::Config(cli) => cli.run().await,
         Workflow::Tls(cli) => cli.run().await,
      }
   }
}
