use anyhow::{anyhow, Context, Result};


#[derive(Clone)]
pub struct Sqlite {
   pool: sqlx::Pool<sqlx::Sqlite>,
}

impl Sqlite {
   pub async fn new(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Sqlite> {
      crate::db::init_ddl(&pool, Self::ddl())
         .await
         .with_context(|| anyhow!("Failed to init ddl"))?;
      Ok(Sqlite { pool: pool.clone() })
   }

   fn ddl() -> &'static [&'static str] {
      &[
         "CREATE TABLE IF NOT EXISTS measurements (read_ts INTEGER  NOT NULL) STRICT;",
         "ALTER TABLE measurements ADD location    TEXT   ;",
         "ALTER TABLE measurements ADD sensor      TEXT   ;",
         "ALTER TABLE measurements ADD index_n     INTEGER;",
         "ALTER TABLE measurements ADD temperature REAL   ;",
         "ALTER TABLE measurements ADD error       TEXT   ;",
      ]
   }
}




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


#[async_trait::async_trait]
impl Db for Sqlite {
   async fn write(&self, row: &common::Measurement) -> Result<()> {
      sqlx::query(
         r#"INSERT INTO measurements (read_ts, temperature, error, location, sensor, index_n)
            VALUES ($1, $2, $3, $4, $5, $6)
         "#,
      )
      .bind(row.read_ts)
      .bind(row.temperature)
      .bind(&row.error)
      .bind(&row.id.location)
      .bind(&row.id.sensor)
      .bind(&row.id.index)
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
         SELECT read_ts, temperature, error, location, sensor, index_n as "index"
         FROM measurements
         WHERE read_ts >= $1 AND read_ts < $2
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
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      let _ = crate::db::init_ddl(&sqlite.pool, Sqlite::ddl()).await;
      Ok(())
   }

   fn ts_ymd(year: i32, month: u32, day: u32) -> common::MicroSecTs {
      use chrono::TimeZone;
      let ts = chrono::Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).earliest().unwrap();
      common::MicroSecTs(ts)
   }

   fn measurement(ts: common::MicroSecTs) -> common::Measurement {
      let id = common::Id {
         location: "tar".to_string(),
         sensor: "ambient".to_string(),
         index: 123,
      };
      let mes = common::Measurement {
         id,
         temperature: Some(26.8),
         error: "error1".to_string(),
         read_ts: ts,
      };
      mes
   }

   #[tokio::test]
   async fn test_read_ts_less_than_start() -> Result<()> {
      let (y, m, d) = (2024, 1, 1);
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
      let res = sqlite.read(ts_ymd(y - 1, m, d), ts_ymd(y - 1, m, d)).await?;

      let expected = Vec::new();
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_read_ts_equals_start() -> Result<()> {
      let (y, m, d) = (2024, 1, 1);
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
      let res = sqlite.read(ts_ymd(y, m, d), ts_ymd(y + 1, m, d)).await?;
      let expected: Vec<common::Measurement> = vec![measurement(ts_ymd(y, m, d))];
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_read_ts_between_start_and_end() -> Result<()> {
      use super::Db;
      let (y, m, d) = (2024, 1, 1);
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
      let res = sqlite.read(ts_ymd(y - 1, m, d), ts_ymd(y + 2, m, d)).await?;
      let expected = vec![measurement(ts_ymd(y, m, d))];
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_read_ts_equals_end() -> Result<()> {
      let (y, m, d) = (2024, 1, 1);
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
      let res = sqlite.read(ts_ymd(y - 1, m, d), ts_ymd(y, m, d)).await?;
      let expected: Vec<common::Measurement> = Vec::new();
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_read_ts_larger_than_end() -> Result<()> {
      let (y, m, d) = (2024, 1, 1);
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
      let res = sqlite.read(ts_ymd(y - 1, m, d), ts_ymd(y - 1, m, d)).await?;
      let expected = Vec::new();
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_read_ts_on_19th() -> Result<()> {
      use chrono::TimeZone;

      let (y, m, _d) = (2024, 1, 1);
      let ts1 = chrono::Utc.with_ymd_and_hms(2024, 1, 19, 23, 59, 58).unwrap();
      let ts2 = chrono::Utc.with_ymd_and_hms(2024, 1, 19, 23, 59, 59).unwrap();
      let ts3 = ts_ymd(y, m, 20);
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.write(&measurement(common::MicroSecTs(ts1))).await?;
      sqlite.write(&measurement(common::MicroSecTs(ts2))).await?;
      sqlite.write(&measurement(ts3)).await?;
      let res = sqlite.read(ts_ymd(y, m, 19), ts3).await?;
      let expected: Vec<common::Measurement> =
         vec![measurement(common::MicroSecTs(ts1)), measurement(common::MicroSecTs(ts2))];
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_read_many() -> Result<()> {
      let (y, m, d) = (2024, 1, 2);
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d + 1))).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d + 2))).await?;
      let res = sqlite.read(ts_ymd(y - 1, m, d - 1), ts_ymd(y + 1, m, d)).await?;
      let expected: Vec<common::Measurement> = vec![
         measurement(ts_ymd(y, m, d)),
         measurement(ts_ymd(y, m, d + 1)),
         measurement(ts_ymd(y, m, d + 2)),
      ];
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_delete_ts_less_than_up_to() -> Result<()> {
      let (y, m, d) = (2024, 1, 1);
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
      sqlite.delete(ts_ymd(y + 1, m, d)).await?;
      let res = sqlite.read(ts_ymd(y - 1, m, d), ts_ymd(y + 1, m, d)).await?;
      let expected = Vec::new();
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_delete_ts_equals_up_to() -> Result<()> {
      let (y, m, d) = (2024, 1, 1);
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d + 1))).await?;
      sqlite.delete(ts_ymd(y, m, d + 1)).await?;
      let res = sqlite.read(ts_ymd(y - 1, m, d), ts_ymd(y + 1, m, d)).await?;
      let expected: Vec<common::Measurement> = vec![measurement(ts_ymd(y, m, d + 1))];
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_delete_ts_larger_than_up_to() -> Result<()> {
      let (y, m, d) = (2024, 1, 1);
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.write(&measurement(ts_ymd(y, m, d))).await?;
      sqlite.delete(ts_ymd(y - 1, m, d)).await?;
      let res = sqlite.read(ts_ymd(y - 1, m, d), ts_ymd(y + 1, m, d)).await?;
      let expected: Vec<common::Measurement> = vec![measurement(ts_ymd(y, m, d))];
      assert_eq!(res, expected);
      Ok(())
   }
}
