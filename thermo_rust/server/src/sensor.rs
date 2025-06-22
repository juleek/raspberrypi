use anyhow::{anyhow, Context, Result};


//
// ===========================================================================================================
// Id

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Id(String);

impl Id {
   const PREFIX: &'static str = "sen_";
   const LEN: &'static usize = &10;
   const NAME: &'static str = "sensor";

   fn new() -> Self { Self(crate::generate_random_id(Id::PREFIX, *Id::LEN)) }

   fn validate(value: &str) -> Result<()> {
      if value.starts_with(Id::PREFIX) == false {
         return Err(anyhow!("Id: {value} does not start with expected prefix {}", Id::PREFIX));
      }
      Ok(())
   }
}

impl TryFrom<String> for Id {
   type Error = anyhow::Error;

   fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
      Id::validate(&value)?;
      Ok(Self(value))
   }
}

impl TryFrom<&str> for Id {
   type Error = anyhow::Error;

   fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
      Id::validate(value)?;
      Ok(Self(value.to_owned()))
   }
}

impl sqlx::Type<sqlx::Sqlite> for Id {
   fn type_info() -> sqlx::sqlite::SqliteTypeInfo { <String as sqlx::Type<sqlx::Sqlite>>::type_info() }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for Id {
   fn encode_by_ref(
      &self,
      buf: &mut <sqlx::Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
   ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
      sqlx::Encode::<sqlx::Sqlite>::encode(&self.0, buf)
   }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for Id {
   fn decode(
      value: <sqlx::Sqlite as sqlx::Database>::ValueRef<'r>,
   ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
      let s: String = sqlx::Decode::<sqlx::Sqlite>::decode(value)?;
      Id::try_from(s).map_err(Into::into)
   }
}


//
// ===========================================================================================================
// Sensor

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Sensor {
   pub id: Id,
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
   async fn get_by_id(&self, id: &Id) -> Result<Option<Sensor>>;
   async fn delete(&self, id: &Id) -> Result<()>;
   async fn update_min(&self, id: &Id, min: f64) -> Result<()>;
   async fn update_name(&self, id: &Id, name: &str) -> Result<()>;
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

   async fn get_by_id(&self, id: &Id) -> Result<Option<Sensor>> {
      let sensor = sqlx::query_as(
         r#"SELECT id, name, location, min
        FROM sensors
        WHERE id = $1
        "#,
      )
      .bind(&id.0)
      .fetch_optional(&self.pool)
      .await?;

      Ok(sensor)
   }

   async fn delete(&self, id: &Id) -> Result<()> {
      sqlx::query(
         r#"DELETE FROM sensors WHERE id = $1
         "#,
      )
      .bind(&id.0)
      .execute(&self.pool)
      .await?;
      Ok(())
   }

   async fn update_min(&self, id: &Id, min: f64) -> Result<()> {
      sqlx::query(
         r#"UPDATE sensors SET min = $1 WHERE id = $2
        "#,
      )
      .bind(&min)
      .bind(&id.0)
      .execute(&self.pool)
      .await?;
      Ok(())
   }

   async fn update_name(&self, id: &Id, name: &str) -> Result<()> {
      sqlx::query(
         r#"UPDATE sensors SET name = $1 WHERE id = $2
        "#,
      )
      .bind(&name)
      .bind(&id.0)
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
   // use super::*;
   // use pretty_assertions::assert_eq;

   // fn sensor(id: String) -> Sensor {
   //    let sensor = Sensor {
   //       id,
   //       name: "sensor".to_string(),
   //       location: "tar".to_string(),
   //       min: Some(5.0),
   //    };
   //    sensor
   // }

   // fn sensor2(id: String) -> Sensor {
   //    let sensor = Sensor {
   //       id,
   //       name: "sensor2".to_string(),
   //       location: "asdf".to_string(),
   //       min: Some(6.0),
   //    };
   //    sensor
   // }

   // #[tokio::test]
   // async fn test_set_update_row_with_same_id() -> Result<()> {
   //    let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
   //    let sqlite = Sqlite::new(&pool).await?;
   //    sqlite.set(&sensor("123tar".to_string())).await?;
   //    sqlite.set(&sensor2("123tar".to_string())).await?;
   //    let res = sqlite.get_by_id("123tar".to_string()).await?;
   //    let expected = Some(sensor2("123tar".to_string()));
   //    assert_eq!(res, expected);
   //    Ok(())
   // }

   // #[tokio::test]
   // async fn test_get_by_id_present_id() -> Result<()> {
   //    let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
   //    let sqlite = Sqlite::new(&pool).await?;
   //    sqlite.set(&sensor("123tar".to_string())).await?;
   //    let res = sqlite.get_by_id("123tar".to_string()).await?;
   //    let expected = Some(sensor("123tar".to_string()));
   //    assert_eq!(res, expected);
   //    Ok(())
   // }

   // #[tokio::test]
   // async fn test_get_by_id_no_found_requested_id() -> Result<()> {
   //    let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
   //    let sqlite = Sqlite::new(&pool).await?;
   //    sqlite.set(&sensor("123tar".to_string())).await?;
   //    let res = sqlite.get_by_id("123".to_string()).await?;
   //    let expected = None;
   //    assert_eq!(res, expected);
   //    Ok(())
   // }

   // #[tokio::test]
   // async fn test_delete_id_is_present() -> Result<()> {
   //    let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
   //    let sqlite = Sqlite::new(&pool).await?;
   //    sqlite.set(&sensor("123tar".to_string())).await?;
   //    sqlite.set(&sensor("456tar".to_string())).await?;
   //    sqlite.delete("123tar".to_string()).await?;
   //    let res = sqlite.get_by_id("123tar".to_string()).await?;
   //    let expected = None;
   //    assert_eq!(res, expected);
   //    Ok(())
   // }

   // #[tokio::test]
   // async fn test_delete_id_is_not_present() -> Result<()> {
   //    let pool = crate::db::Location::create_pool(&crate::db::Location::Memory).await?;
   //    let sqlite = Sqlite::new(&pool).await?;
   //    sqlite.set(&sensor("123tar".to_string())).await?;
   //    sqlite.delete("456tar".to_string()).await?;
   //    let res = sqlite.get_by_id("123tar".to_string()).await?;
   //    let expected = Some(sensor("123tar".to_string()));
   //    assert_eq!(res, expected);
   //    Ok(())
   // }
}
