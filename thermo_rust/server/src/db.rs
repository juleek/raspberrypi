use anyhow::{anyhow, Context, Result};


//
// ===========================================================================================================
// Trait

#[async_trait::async_trait]
pub trait Db {
   async fn write(&self, row: &common::Measurement) -> Result<()>;
   async fn read(
      &self,
      start: common::MicroSecTs,
      end: common::MicroSecTs,
   ) -> Result<Vec<common::Measurement>>;
   async fn delete(&self, up_to: common::MicroSecTs) -> Result<()>;
}




//
// ===========================================================================================================
// Sqlite implementation

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

pub struct Sqlite {
   pool: sqlx::Pool<sqlx::Sqlite>,
}

impl Sqlite {
   pub async fn new(location: &Location) -> Result<Sqlite> {
      let opts = match location {
         Location::Memory => {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let random = (0..4).map(|_| rng.gen_range(b'A'..b'z') as u8).collect();
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
      Self::init_ddl(&pool).await.with_context(|| anyhow!("Failed to init ddl"))?;
      Ok(Sqlite { pool })
   }

   fn ddl() -> &'static [&'static str] {
      &[
         "CREATE TABLE IF NOT EXISTS measurements (read_ts INTEGER  NOT NULL) STRICT;",
         "ALTER TABLE measurements ADD sensor      text     NOT NULL;",
         "ALTER TABLE measurements ADD temperature real;",
         "ALTER TABLE measurements ADD error      text;",
      ]
   }
   async fn init_ddl(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
      for sql in Self::ddl() {
         println!("executing ddl {sql}");
         sqlx::query(sql).execute(pool).await.map(|_| ()).or_else(ignore_duplicate_column_name)?;
      }
      Ok(())
   }
}

#[async_trait::async_trait]
impl Db for Sqlite {
   async fn write(&self, row: &common::Measurement) -> Result<()> {
      sqlx::query(
         r#"INSERT INTO measurements (read_ts, sensor, temperature, error)
            VALUES ($1, $2, $3, $4)
         "#,
      )
      .bind(row.read_ts)
      .bind(&row.sensor)
      .bind(row.temperature)
      .bind(&row.error)
      .execute(&self.pool)
      .await?;
      Ok(())
   }

   async fn read(
      &self,
      start: common::MicroSecTs,
      end: common::MicroSecTs,
   ) -> Result<Vec<common::Measurement>> {
      let measurements = sqlx::query_as(
         r#"
         SELECT read_ts, sensor, temperature, error
         FROM measurements
         WHERE read_ts >= $1 AND read_ts <= $2
         ORDER BY read_ts
         "#,
      )
      .bind(start)
      .bind(end)
      .fetch_all(&self.pool)
      .await?;

      Ok(measurements)

   }

   async fn delete(&self, up_to: common::MicroSecTs) -> Result<()> {
      sqlx::query(
         r#"DELETE FROM measurements WHERE read_ts < $1
         "#,
      )
      .bind(up_to)
      .execute(&self.pool)
      .await?;
      Ok(())
   }
}


//
// ===========================================================================================================
// Tests


#[cfg(test)]
mod tests {
   use super::*;
   use pretty_assertions::assert_eq;

   #[tokio::test]
   async fn test_init_ddl_is_idempotent() -> Result<()> {
      let sqlite = Sqlite::new(&Location::Memory).await?;
      Sqlite::init_ddl(&sqlite.pool).await;
      Ok(())
   }

   fn ts_ymd(year: i32, month: u32, day: u32) -> common::MicroSecTs {
      use chrono::TimeZone;
      let ts = chrono::Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).earliest().unwrap();
      common::MicroSecTs(ts)
   }

   fn measurement(ts: common::MicroSecTs) -> common::Measurement {
      let mes = common::Measurement {
         sensor: "ambient".to_string(),
         temperature: Some(26.8),
         error: "error1".to_string(),
         read_ts: ts,
      };
      println!("asdf mes {mes:?}");
      mes
   }

   // #[tokio::test]
   // async fn test_read_ts_less_than_start() -> Result<()> {
   //    let (y, m, d) = (2024, 1, 1);
   //    let sqlite = Sqlite::new(&Location::Memory).await?;
   //    sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
   //    let res = sqlite.read(ts_ymd(y - 1, m, d), ts_ymd(y - 1, m, d)).await?;

   //    let expected = Vec::new();
   //    assert_eq!(res, expected);
   //    Ok(())
   // }

   // #[tokio::test]
   // async fn test_read_ts_equals_start() -> Result<()> {
   //    let (y, m, d) = (2024, 1, 1);
   //    let sqlite = Sqlite::new(&Location::Memory).await?;
   //    sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
   //    let res = sqlite.read(ts_ymd(y, m, d), ts_ymd(y + 1, m, d)).await?;
   //    let expected: Vec<common::Measurement> = vec![measurement(ts_ymd(y, m, d))];
   //    assert_eq!(res, expected);
   //    Ok(())
   // }

   #[tokio::test]
   async fn test_read_ts_between_start_and_end() -> Result<()> {
      use super::Db;
      let (y, m, d) = (2024, 1, 1);
      let sqlite = Sqlite::new(&Location::Memory).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
      let res = sqlite.read(ts_ymd(y - 1, m, d), ts_ymd(y + 2, m, d)).await?;
      let expected  = vec![measurement(ts_ymd(y, m, d))];
      assert_eq!(res, expected);
      Ok(())
   }

   // #[tokio::test]
   // async fn test_read_ts_equals_end() -> Result<()> {
   //    let (y, m, d) = (2024, 1, 1);
   //    let sqlite = Sqlite::new(&Location::Memory).await?;
   //    sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
   //    let res = sqlite.read(ts_ymd(y - 1, m, d), ts_ymd(y, m, d)).await?;
   //    let expected: Vec<common::Measurement> = vec![measurement(ts_ymd(y, m, d))];
   //    assert_eq!(res, expected);
   //    Ok(())
   // }

   // #[tokio::test]
   // async fn test_read_ts_larger_than_end() -> Result<()> {
   //    let (y, m, d) = (2024, 1, 1);
   //    let sqlite = Sqlite::new(&Location::Memory).await?;
   //    sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
   //    let res = sqlite.read(ts_ymd(y - 1, m, d), ts_ymd(y - 1, m, d)).await?;
   //    let expected = Vec::new();
   //    assert_eq!(res, expected);
   //    Ok(())
   // }

   // #[tokio::test]
   // async fn test_read_many() -> Result<()> { Ok(()) }

   // #[tokio::test]
   // async fn test_delete_ts_less_than_up_to() -> Result<()> { Ok(()) }

   // #[tokio::test]
   // async fn test_delete_ts_equals_up_to() -> Result<()> { Ok(()) }

   // #[tokio::test]
   // async fn test_delete_ts_larger_than_up_to() -> Result<()> { Ok(()) }
}
