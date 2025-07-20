use anyhow::{Context, Result, anyhow};




#[derive(clap::Parser, Debug)]
pub struct SensorGenIdOpts {
   /// How many sensors IDs do you need?
   #[arg(short, default_value_t = 1)]
   n: u32,
}



impl SensorGenIdOpts {
   pub async fn run(&self) -> Result<()> {
      for _ in 0..self.n {
         let res = common::SensorId::new();
         println!("{res}");
      }
      Ok(())
   }
}



// ===========================================================================================================

async fn create_sqlite(path: &str) -> Result<crate::sensor::Sqlite> {
   let path = std::path::PathBuf::from(path);
   let pool = crate::db::Location::create_pool(&crate::db::Location::Path(path)).await?;
   crate::sensor::Sqlite::new(&pool).await
}


#[derive(clap::Parser, Debug)]
pub struct SensorAddOpts {
   #[arg(long)]
   db_path: String,

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
   pub async fn run(&self) -> Result<()> {
      let sqlite = create_sqlite(&self.db_path).await?;

      let sensor = crate::sensor::Sensor {
         id: self.id.clone().try_into()?,
         name: self.name.clone(),
         location: self.location.clone(),
         min: self.min,
      };
      use crate::sensor::Db;
      sqlite.add(&sensor).await.with_context(|| anyhow!("Failed to add {sensor:?}"))?;
      Ok(())
   }
}

// ===========================================================================================================



#[derive(clap::Parser, Debug)]
pub struct SensorUpdateOpts {
   #[arg(long)]
   db_path: String,

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
   pub async fn run(&self) -> Result<()> {
      let sqlite = create_sqlite(&self.db_path).await?;

      use crate::sensor::Db;
      let id: common::SensorId = self.id.clone().try_into()?;
      if let Some(min) = self.min {
         sqlite
            .update_min(&id, min)
            .await
            .with_context(|| anyhow!("Failed to update min of {id}"))?;
      }
      if let Some(name) = &self.name {
         sqlite
            .update_name(&id, name)
            .await
            .with_context(|| anyhow!("Failed to update name of {id}"))?;
      }
      Ok(())
   }
}


// ===========================================================================================================

#[derive(clap::Subcommand, Debug)]
pub enum Workflow {
   SensorGenId(SensorGenIdOpts),
   SensorAdd(SensorAddOpts),
   SensorUpdate(SensorUpdateOpts),
   // TODO: list sensors
}



/// Config management operations, such as managing sensors (names, thresholds, etc...)
#[derive(clap::Parser, Debug)]
pub struct Cli {
   #[command(subcommand)]
   workflow: Workflow,
}

impl Cli {
   pub async fn run(&self) -> Result<()> {
      // create db instance
      match &self.workflow {
         Workflow::SensorGenId(opts) => opts.run().await,
         Workflow::SensorAdd(opts) => opts.run().await,
         Workflow::SensorUpdate(opts) => opts.run().await,
      }
   }
}
