use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Sensor {
   pub id: String,
   pub name: String,
   pub location: String,
   pub min: Option<f64>,
}

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
         "CREATE TABLE IF NOT EXISTS sensors (id TEXT PRIMARY KEY) STRICT;",
         "ALTER TABLE sensors ADD name        TEXT;",
         "ALTER TABLE sensors ADD location    TEXT;",
         "ALTER TABLE sensors ADD min         REAL;",
      ]
   }
}


//
// ===========================================================================================================
// Trait

#[async_trait::async_trait]
pub trait Db {
   async fn set(&self, sensor: &Sensor) -> Result<()>;
   async fn get_by_id(&self, id: String) -> Result<Option<Sensor>>;
   async fn delete(&self, id: String) -> Result<()>;
}


#[async_trait::async_trait]
impl Db for Sqlite {
   async fn set(&self, row: &Sensor) -> Result<()> {
      sqlx::query(
        r#"INSERT INTO sensors (id, name, location, min)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT(id) DO UPDATE SET
              name = $2,
              location = $3,
              min = $4
        "#,
    )
      .bind(&row.id)
      .bind(&row.name)
      .bind(&row.location)
      .bind(&row.min)
      .execute(&self.pool)
      .await?;
      Ok(())
   }

   async fn get_by_id(&self, id: String) -> Result<Option<Sensor>> {
      let sensor = sqlx::query_as(
         r#"SELECT id, name, location, min
        FROM sensors
        WHERE id = $1
        "#,
      )
      .bind(id)
      .fetch_optional(&self.pool)
      .await?;

      Ok(sensor)
   }

   async fn delete(&self, id: String) -> Result<()> {
      sqlx::query(
         r#"DELETE FROM sensors WHERE id = $1
         "#,
      )
      .bind(id)
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

   fn sensor(id: String) -> Sensor {
      let sensor = Sensor {
         id,
         name: "sensor".to_string(),
         location: "tar".to_string(),
         min: Some(5.0),
      };
      sensor
   }

   fn sensor2(id: String) -> Sensor {
      let sensor = Sensor {
         id,
         name: "sensor2".to_string(),
         location: "asdf".to_string(),
         min: Some(6.0),
      };
      sensor
   }

   #[tokio::test]
   async fn test_set_update_row_with_same_id() -> Result<()> {
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.set(&sensor("123tar".to_string())).await?;
      sqlite.set(&sensor2("123tar".to_string())).await?;
      let res = sqlite.get_by_id("123tar".to_string()).await?;
      let expected = Some(sensor2("123tar".to_string()));
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_get_by_id_present_id() -> Result<()> {
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.set(&sensor("123tar".to_string())).await?;
      let res = sqlite.get_by_id("123tar".to_string()).await?;
      let expected = Some(sensor("123tar".to_string()));
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_get_by_id_no_found_requested_id() -> Result<()> {
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.set(&sensor("123tar".to_string())).await?;
      let res = sqlite.get_by_id("123".to_string()).await?;
      let expected = None;
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_delete_id_is_present() -> Result<()> {
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.set(&sensor("123tar".to_string())).await?;
      sqlite.set(&sensor("456tar".to_string())).await?;
      sqlite.delete("123tar".to_string()).await?;
      let res = sqlite.get_by_id("123tar".to_string()).await?;
      let expected = None;
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_delete_id_is_not_present() -> Result<()> {
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      sqlite.set(&sensor("123tar".to_string())).await?;
      sqlite.delete("456tar".to_string()).await?;
      let res = sqlite.get_by_id("123tar".to_string()).await?;
      let expected = Some(sensor("123tar".to_string()));
      assert_eq!(res, expected);
      Ok(())
   }
}
