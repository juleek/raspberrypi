use anyhow::Result;


#[derive(clap::Parser, Debug)]
struct SensorAddOpts {
   /// location
   #[arg(long)]
   location: String,

   /// id
   #[arg(long)]
   id: String,

   /// name
   #[arg(long)]
   name: String,

   /// name
   #[arg(long)]
   min: String,
}

impl SensorAddOpts {
   pub async fn run(&self, db_path: &str) -> Result<()> {
      // create db instance (meta table)
      // insert a new record
      todo!()
   }
}

// ===========================================================================================================



#[derive(clap::Parser, Debug)]
struct SensorUpdateOpts {
   /// id
   #[arg(long)]
   id: String,

   /// name
   #[arg(long)]
   min: String,
}

impl SensorUpdateOpts {
   pub async fn run(&self, db_path: &str) -> Result<()> {
      // create db instance (meta table)
      // upsert
      todo!()
   }
}


// ===========================================================================================================

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
   SensorAdd(SensorAddOpts),
   SensorUpdate(SensorUpdateOpts),
}



/// Config management operations, such as managing sensors (names, thresholds, etc...)
#[derive(clap::Parser, Debug)]
pub struct Cli {
   #[arg(long)]
   db_path: String,

   #[command(subcommand)]
   command: Commands,
}

impl Cli {
   pub async fn run(&self) -> Result<()> {
      // create db instance
      match &self.command {
         Commands::SensorAdd(opts) => opts.run(&self.db_path).await,
         Commands::SensorUpdate(opts) => opts.run(&self.db_path).await,
      }
   }
}
