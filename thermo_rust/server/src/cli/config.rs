use anyhow::Result;

#[derive(clap::Parser, Debug)]
pub struct SensorAddOpts {
   /// id
   #[arg(long)]
   id: String,

   /// location
   #[arg(long)]
   location: String,

   /// name
   #[arg(long)]
   name: String,

   /// name
   #[arg(long)]
   min: f64,
}



impl SensorAddOpts {
   pub async fn run(&self, _sqlite: crate::sensor::Sqlite) -> Result<()> {
      todo!()
      // let sensor = crate::db::sensor::Sensor{
      //    id: self.id.clone(),
      //    name: self.name.clone(),
      //    location: self.location.clone(),
      //    min: self.min
      // };
      // use crate::db::sensor::Db;
      // sqlite.set(&sensor);
      // Ok(())

   }
}

// ===========================================================================================================



#[derive(clap::Parser, Debug)]
pub struct SensorUpdateOpts {
   /// id
   #[arg(long)]
   id: String,

   /// name
   #[arg(long)]
   min: Option<f64>,

   /// name
   #[arg(long)]
   name: Option<String>,
}

impl SensorUpdateOpts {
   pub async fn run(&self, _sqlite: crate::sensor::Sqlite) -> Result<()> {
      todo!()
      // let sensor = crate::db::sensor::Sensor{
      //    id: self.id.clone(),
      //    name: self.name.clone(),
      //    location: self.location.clone(),
      //    min: self.min
      // };
      // use crate::db::sensor::Db;
      // sqlite.set(&sensor);
      // Ok(())



      //updatemin
      //updatename
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
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = crate::sensor::Sqlite::new(&pool).await?;
      // create db instance
      match &self.command {
         Commands::SensorAdd(opts) => opts.run(sqlite).await,
         Commands::SensorUpdate(opts) => opts.run(sqlite).await,
      }
   }
}
