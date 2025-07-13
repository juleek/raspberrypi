use anyhow::{anyhow, Context, Result};


//
// ===========================================================================================================
// Sensor

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Sensor {
   pub id: common::SensorId,
   pub name: String,
   pub location: String,
   pub min: f64,
}


//
// ===========================================================================================================
// Db

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

#[async_trait::async_trait]
pub trait Db {
   async fn add(&self, sensor: &Sensor) -> Result<()>;
   async fn get_by_id(&self, id: &common::SensorId) -> Result<Option<Sensor>>;
   async fn delete(&self, id: &common::SensorId) -> Result<()>;
   async fn update_min(&self, id: &common::SensorId, min: f64) -> Result<()>;
   async fn update_name(&self, id: &common::SensorId, name: &str) -> Result<()>;
}


#[async_trait::async_trait]
impl Db for Sqlite {
   async fn add(&self, row: &Sensor) -> Result<()> {
      sqlx::query(
         r#"INSERT INTO sensors (id, name, location, min)
           VALUES ($1, $2, $3, $4)
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

   async fn get_by_id(&self, id: &common::SensorId) -> Result<Option<Sensor>> {
      let sensor = sqlx::query_as(
         r#"SELECT id, name, location, min
        FROM sensors
        WHERE id = $1
        "#,
      )
      .bind(id.clone())
      .fetch_optional(&self.pool)
      .await?;

      Ok(sensor)
   }

   async fn delete(&self, id: &common::SensorId) -> Result<()> {
      sqlx::query(
         r#"DELETE FROM sensors WHERE id = $1
         "#,
      )
      .bind(id.clone())
      .execute(&self.pool)
      .await?;
      Ok(())
   }

   async fn update_min(&self, id: &common::SensorId, min: f64) -> Result<()> {
      sqlx::query(
         r#"UPDATE sensors SET min = $1 WHERE id = $2
        "#,
      )
      .bind(&min)
      .bind(id.clone())
      .execute(&self.pool)
      .await?;
      Ok(())
   }

   async fn update_name(&self, id: &common::SensorId, name: &str) -> Result<()> {
      sqlx::query(
         r#"UPDATE sensors SET name = $1 WHERE id = $2
        "#,
      )
      .bind(&name)
      .bind(id.clone())
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

   fn s_id_name(id: &common::SensorId, name: String) -> Sensor {
      let sensor = Sensor {
         id: id.clone(),
         name,
         location: "tar".to_string(),
         min: 5.0,
      };
      sensor
   }

   fn s_id_min(id: &common::SensorId, min: f64) -> Sensor {
      let sensor = Sensor {
         id: id.clone(),
         name: "sensor2".to_string(),
         location: "asdf".to_string(),
         min,
      };
      sensor
   }

   fn s_id(id: &common::SensorId) -> Sensor {
      let sensor = Sensor {
         id: id.clone(),
         name: "sensor2".to_string(),
         location: "asdf".to_string(),
         min: 5.0,
      };
      sensor
   }

   #[tokio::test]
   async fn test_set_update_name_with_same_id() -> Result<()> {
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      let id = common::SensorId::new();
      sqlite.add(&s_id_name(&id, "sensor".to_string())).await?;
      sqlite.update_name(&id, &"sensor2").await?;
      let res = sqlite.get_by_id(&id).await?;
      let expected = Some(s_id_name(&id, "sensor2".to_string()));
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_set_update_min_with_same_id() -> Result<()> {
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      let id = common::SensorId::new();
      sqlite.add(&s_id_min(&id, 6.0)).await?;
      sqlite.update_min(&id, 10.0).await?;
      let res = sqlite.get_by_id(&id).await?;
      let expected = Some(s_id_min(&id, 10.0));
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_get_by_id_present_id() -> Result<()> {
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      let id = common::SensorId::new();
      sqlite.add(&s_id(&id)).await?;
      let res = sqlite.get_by_id(&id).await?;
      let expected = Some(s_id(&id));
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_get_by_id_no_found_requested_id() -> Result<()> {
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      let id = common::SensorId::new();
      sqlite.add(&s_id(&id)).await?;
      let id_new = common::SensorId::new();
      let res = sqlite.get_by_id(&id_new).await?;
      let expected = None;
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_delete_id_is_present() -> Result<()> {
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      let id = common::SensorId::new();
      sqlite.add(&s_id(&id)).await?;
      let id2 = common::SensorId::new();
      sqlite.add(&s_id(&id2)).await?;
      sqlite.delete(&id).await?;
      let res = sqlite.get_by_id(&id).await?;
      let expected = None;
      assert_eq!(res, expected);
      Ok(())
   }

   #[tokio::test]
   async fn test_delete_id_is_not_present() -> Result<()> {
      let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
      let sqlite = Sqlite::new(&pool).await?;
      let id = common::SensorId::new();
      sqlite.add(&s_id(&id)).await?;
      let id2 = common::SensorId::new();
      sqlite.delete(&id2).await?;
      let res = sqlite.get_by_id(&id).await?;
      let expected = Some(s_id(&id));
      assert_eq!(res, expected);
      Ok(())
   }
}
