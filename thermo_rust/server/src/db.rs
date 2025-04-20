use anyhow::{anyhow, Context, Ok, Result};


//
// ===========================================================================================================
// Trait

#[async_trait::async_trait]
pub trait Db {
   async fn write(&self, row: &common::Measurement) -> Result<()>;
   async fn read(
      &self,
      start: chrono::DateTime<chrono::Utc>,
      end: chrono::DateTime<chrono::Utc>,
   ) -> Result<Vec<common::Measurement>>;
   async fn delete(&self, up_to: chrono::DateTime<chrono::Utc>) -> Result<()>;
}


//
// ===========================================================================================================
// Sqlite implementation


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
         Location::Memory => sqlx::sqlite::SqliteConnectOptions::new()
            .in_memory(true)
            .shared_cache(true)
            .filename("file:"),
         Location::Path(p) => sqlx::sqlite::SqliteConnectOptions::new().filename(p),
      };
      // use std::str::FromStr;
      // let o = sqlx::sqlite::SqliteConnectOptions::from_str("file::memory:?cache=shared")?;
      // println!("{o:#?}");
      // let pool = sqlx::sqlite::SqlitePool::connect("file::memory:?cache=shared").await?;

      let pool = sqlx::sqlite::SqlitePool::connect_with(opts.clone())
         .await
         .with_context(|| anyhow!("Failed to create sqlite pool with optons: {opts:?}"))?;
      Self::init_ddl(&pool).await.with_context(|| anyhow!("Failed to init ddl"))?;
      Ok(Sqlite { pool })
   }

   fn ddl() -> &'static [&'static str] {
      &[
         "CREATE TABLE IF NOT EXISTS measurements (read_ts INTEGER  NOT NULL);",
         // "ALTER TABLE measurements ADD sensor      text     NOT NULL;",
         "ALTER TABLE measurements ADD temperature real;",
         "ALTER TABLE measurements ADD error      text;",
      ]
   }
   async fn init_ddl(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<()> {
      // let mut conn = pool.acquire().await?;
      for sql in Self::ddl() {
         println!("executing ddl {sql}");
         sqlx::query(sql)
            // .execute(&mut *conn)
            .execute(pool)
            .await
            .with_context(|| anyhow!("Error while executing ddl: {sql}"))?;
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
      .bind(row.read_ts.timestamp_micros())
      .bind(&row.sensor)
      .bind(row.temperature)
      .bind(&row.error)
      .execute(&self.pool)
      .await?;
      Ok(())
   }

   async fn read(
      &self,
      start: chrono::DateTime<chrono::Utc>,
      end: chrono::DateTime<chrono::Utc>,
   ) -> Result<Vec<common::Measurement>> {
      let measurements = sqlx::query_as(
         r#"
         SELECT read_ts, sensor, temperature, error
         FROM measurements
         WHERE read_ts >= $1 AND read_ts <= $2
         ORDER BY read_ts
         "#,
      )
      .bind(start.timestamp_micros())
      .bind(end.timestamp_micros())
      .fetch_all(&self.pool)
      .await?;

      Ok(measurements)
   }

   async fn delete(&self, up_to: chrono::DateTime<chrono::Utc>) -> Result<()> {
      sqlx::query(
         r#"DELETE FROM measurements WHERE read_ts < $1
         "#,
      )
      .bind(up_to.timestamp_micros())
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
      Sqlite::init_ddl(&sqlite.pool).await?;
      Ok(())
   }
}
