use anyhow::Result;

#[derive(clap::Parser, Debug)]
pub struct Cli {
   #[command(subcommand)]
   workflow: Workflow,
}
impl Cli {
   pub async fn run(&self) -> Result<()> { self.workflow.run().await }
}


#[derive(clap::Subcommand, Debug)]
pub enum Workflow {
   CA(common::tls::GenCaOpts),
   Server(common::tls::GenServerOpts),
   Client(common::tls::GenClientOpts),
}

impl Workflow {
   pub async fn run(&self) -> Result<()> {
      match self {
         Workflow::CA(cli) => cli.run().await,
         Workflow::Server(cli) => cli.run().await,
         Workflow::Client(cli) => cli.run().await,
      }
   }
}
