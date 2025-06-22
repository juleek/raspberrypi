pub mod measurement;
pub mod sensor;
use anyhow::{anyhow, Context, Result};


fn ignore_duplicate_column_name(err_sqlx: sqlx::Error) -> Result<(), sqlx::Error> {
   // let Some(err) = err else { return false };
   let sqlx::Error::Database(err) = &err_sqlx else {
      return Err(err_sqlx);
   };
   let Some(err) = err.try_downcast_ref::<sqlx::sqlite::SqliteError>() else {
      return Err(err_sqlx);
   };
   use sqlx::error::DatabaseError;
   let Some(code) = err.code() else {
      return Err(err_sqlx);
   };
   if code == "1" {
      Ok(())
   } else {
      Err(err_sqlx)
   }
}

pub enum Location {
   Memory,
   Path(std::path::PathBuf),
}

impl Location {
   pub async fn create_pool(&self) -> Result<sqlx::Pool<sqlx::Sqlite>> {
      let opts = match self {
         Location::Memory => {
            use rand::Rng;
            let mut rng = rand::rng();
            let random = (0..4).map(|_| rng.random_range(b'A'..b'z') as u8).collect();
            let random = String::from_utf8(random).unwrap();
            sqlx::sqlite::SqliteConnectOptions::new()
               .in_memory(true)
               .shared_cache(true)
               .filename(format!("file:{random}"))
         }
         Location::Path(p) => sqlx::sqlite::SqliteConnectOptions::new().filename(p),
      };

      let pool = sqlx::sqlite::SqlitePool::connect_with(opts.clone())
         .await
         .with_context(|| anyhow!("Failed to create sqlite pool with optons: {opts:?}"))?;

      Ok(pool)
   }
}

async fn init_ddl(pool: &sqlx::Pool<sqlx::Sqlite>, ddls: &[&str]) -> Result<()> {
   for sql in ddls {
      println!("executing ddl {sql}");
      sqlx::query(sql)
         .execute(pool)
         .await
         .map(|_| ())
         .or_else(crate::db::ignore_duplicate_column_name)?;
   }
   Ok(())
}
