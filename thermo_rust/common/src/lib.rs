pub mod pb;
use anyhow::{anyhow, Result};


// ===========================================================================================================


#[derive(Debug, Clone, PartialEq, Copy)]
pub struct MicroSecTs(pub chrono::DateTime<chrono::Utc>);

impl std::ops::Deref for MicroSecTs {
   type Target = chrono::DateTime<chrono::Utc>;
   fn deref(&self) -> &Self::Target { &self.0 }
}

impl sqlx::Type<sqlx::sqlite::Sqlite> for MicroSecTs {
   fn type_info() -> sqlx::sqlite::SqliteTypeInfo { <i64 as sqlx::Type<sqlx::sqlite::Sqlite>>::type_info() }
}

impl sqlx::Encode<'_, sqlx::sqlite::Sqlite> for MicroSecTs {
   fn encode_by_ref(
      &self,
      args: &mut Vec<sqlx::sqlite::SqliteArgumentValue>,
   ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
      let seconds = self.0.timestamp();
      let micros = self.0.timestamp_subsec_micros() as i64;
      let total_micros = seconds * 1_000_000 + micros;
      args.push(sqlx::sqlite::SqliteArgumentValue::Int64(total_micros));
      Ok(sqlx::encode::IsNull::No)
   }
}

impl sqlx::Decode<'_, sqlx::sqlite::Sqlite> for MicroSecTs {
   fn decode(value: sqlx::sqlite::SqliteValueRef) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
      let total_micros = <i64 as sqlx::Decode<'_, sqlx::sqlite::Sqlite>>::decode(value)?;
      let seconds = total_micros / 1_000_000;
      let micros = total_micros % 1_000_000;
      let nanos = (micros * 1_000) as u32;
      let dt = chrono::DateTime::from_timestamp(seconds, nanos).ok_or_else(|| "Invalid timestamp")?;
      Ok(MicroSecTs(dt))
   }
}

impl From<chrono::DateTime<chrono::Utc>> for crate::MicroSecTs {
   fn from(ts: chrono::DateTime<chrono::Utc>) -> Self { MicroSecTs(ts) }
}

// ===========================================================================================================

fn proto_timestamp_to_chrono(proto: prost_types::Timestamp) -> Result<chrono::DateTime<chrono::Utc>> {
   let chrono_ts = chrono::DateTime::from_timestamp(proto.seconds, proto.nanos as u32)
      .map_or_else(|| Err(anyhow!("Failed to convert proto: {proto} to chrono")), Ok)?;
   Ok(chrono_ts)
}

fn chrono_timestamp_to_proto(ts: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
   prost_types::Timestamp {
      seconds: ts.timestamp(),
      nanos: ts.timestamp_subsec_nanos() as i32,
   }
}


// ===========================================================================================================


#[derive(Debug, Clone, PartialEq, Eq, Hash, sqlx::FromRow)]
pub struct Id {
   pub location: String,
   pub sensor: String,
   pub index: i64,
}

impl Id {
   pub fn new(location: impl Into<String>, sensor: impl Into<String>) -> Self {
      Id {
         location: location.into(),
         sensor: sensor.into(),
         index: chrono::Utc::now().timestamp_nanos_opt().unwrap(),
      }
   }

   pub fn next(&mut self) { self.index += 1; }
}

impl From<Id> for crate::pb::Id {
   fn from(id: Id) -> Self {
      Self {
         location: id.location,
         sensor: id.sensor,
         index: id.index,
      }
   }
}

impl From<crate::pb::Id> for Id {
   fn from(id: crate::pb::Id) -> Self {
      Self {
         location: id.location,
         sensor: id.sensor,
         index: id.index,
      }
   }
}

// ===========================================================================================================

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Measurement {
   #[sqlx(flatten)]
   pub id: Id,
   pub read_ts: MicroSecTs,
   pub temperature: Option<f64>,
   pub error: String,
}

pub type Rx = tokio::sync::mpsc::Receiver<Measurement>;
pub type Tx = tokio::sync::mpsc::Sender<Measurement>;

impl From<Measurement> for crate::pb::Measurement {
   fn from(value: Measurement) -> Self {
      Self {
         id: Some(value.id.into()),
         temperature: value.temperature,
         error: value.error,
         read_ts: Some(chrono_timestamp_to_proto(*value.read_ts)),
      }
   }
}

impl TryFrom<crate::pb::Measurement> for Measurement {
   type Error = anyhow::Error;

   fn try_from(proto: crate::pb::Measurement) -> Result<Self, Self::Error> {
      let read_ts = proto.read_ts.ok_or_else(|| anyhow!("read_ts is None in {proto:?}"))?;
      let id = proto.id.ok_or_else(|| anyhow!("error with id"))?;
      let res = Self {
         id: id.into(),
         temperature: proto.temperature,
         error: proto.error,
         read_ts: proto_timestamp_to_chrono(read_ts)?.into(),
      };
      Ok(res)
   }
}


// ===========================================================================================================


pub fn init_logger(log_level: &str) {
   let mut builder = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level));
   builder.format_timestamp_micros();
   builder.init();
}


//
// ===========================================================================================================
// Tests

#[cfg(test)]
mod tests {
   use super::*;
   use anyhow::Result;
   use pretty_assertions::assert_eq;

   #[test]
   fn test_measurement_proto_conversion() -> Result<()> {
      let ts = chrono::Utc::now();
      let expected: Measurement = Measurement {
         id: Id.new(),
         read_ts: MicroSecTs(ts),
         temperature: Some(26.8),
         error: "error1".to_string(),
      };
      let proto: crate::pb::Measurement = expected.clone().into();
      assert_eq!(
         proto,
         crate::pb::Measurement {
            id: expected.id,
            read_ts: Some(chrono_timestamp_to_proto(ts)),
            temperature: Some(26.8),
            error: "error1".to_string(),
         }
      );

      let actual: Measurement = proto.try_into()?;
      assert_eq!(actual, expected);

      Ok(())
   }
}
