pub mod pb;
pub mod tls;
use anyhow::{anyhow, Context, Result};


pub fn generate_random_string(prefix: &str, len: usize) -> String {
   use rand::Rng;
   let mut rng = rand::rng();
   let mut res = prefix.to_owned();
   for _ in 0..len {
      res.push(rng.sample(rand::distr::Alphanumeric) as char);
   }
   res
}


// ===========================================================================================================
// MicroSecTs


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


//
// ===========================================================================================================
// SensorId

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, derive_more::Display, Hash)]
pub struct SensorId(String);

impl SensorId {
   const PREFIX: &'static str = "sen_";
   const LEN: &'static usize = &10;
   const NAME: &'static str = "sensor";

   pub fn new() -> Self { Self(generate_random_string(SensorId::PREFIX, *SensorId::LEN)) }

   pub fn validate(value: &str) -> Result<()> {
      if value.starts_with(SensorId::PREFIX) == false {
         return Err(anyhow!(
            "{} id: {value} does not start with expected prefix {}",
            SensorId::NAME,
            SensorId::PREFIX
         ));
      }
      Ok(())
   }
}

impl TryFrom<String> for SensorId {
   type Error = anyhow::Error;

   fn try_from(value: String) -> Result<Self, Self::Error> {
      SensorId::validate(&value)?;
      Ok(Self(value))
   }
}

impl TryFrom<&str> for SensorId {
   type Error = anyhow::Error;

   fn try_from(value: &str) -> Result<Self, Self::Error> {
      SensorId::validate(value)?;
      Ok(Self(value.to_owned()))
   }
}

impl From<SensorId> for String {
   fn from(sid: SensorId) -> Self { sid.0 }
}

impl sqlx::Type<sqlx::Sqlite> for SensorId {
   fn type_info() -> sqlx::sqlite::SqliteTypeInfo { <String as sqlx::Type<sqlx::Sqlite>>::type_info() }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for SensorId {
   fn encode_by_ref(
      &self,
      buf: &mut <sqlx::Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
   ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
      sqlx::Encode::<sqlx::Sqlite>::encode(&self.0, buf)
   }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for SensorId {
   fn decode(value: <sqlx::Sqlite as sqlx::Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
      let s: String = sqlx::Decode::<sqlx::Sqlite>::decode(value)?;
      SensorId::try_from(s).map_err(Into::into)
   }
}


// ===========================================================================================================
// MeasurementId


#[derive(Debug, Clone, PartialEq, Eq, Hash, sqlx::FromRow)]
pub struct MeasurementId {
   pub sensor_id: SensorId,
   pub index: i64,
}

impl MeasurementId {
   pub fn new(sensor_id: &SensorId) -> Self {
      MeasurementId {
         sensor_id: sensor_id.clone(),
         index: chrono::Utc::now().timestamp_nanos_opt().unwrap(),
      }
   }

   pub fn next(&mut self) { self.index += 1; }
}

impl From<MeasurementId> for crate::pb::MeasurementId {
   fn from(id: MeasurementId) -> Self {
      Self {
         sensor_id: id.sensor_id.into(),
         index: id.index,
      }
   }
}

impl TryFrom<crate::pb::MeasurementId> for MeasurementId {
   type Error = anyhow::Error;
   fn try_from(id: crate::pb::MeasurementId) -> Result<Self, Self::Error> {
      Ok(Self {
         sensor_id: id.sensor_id.try_into()?,
         index: id.index,
      })
   }
}

// ===========================================================================================================
// Measurement

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Measurement {
   #[sqlx(flatten)]
   pub id: MeasurementId,
   pub read_ts: MicroSecTs,
   pub temperature: Option<f64>,
   pub error: String,
}

impl Measurement {
   pub fn from_ok(id: &MeasurementId, temperature: f64, read_ts: MicroSecTs) -> Self {
      Self {
         id: id.clone(),
         temperature: Some(temperature),
         error: Default::default(),
         read_ts,
      }
   }
   pub fn from_err(id: &MeasurementId, error: impl Into<String>, read_ts: MicroSecTs) -> Self {
      Self {
         id: id.clone(),
         temperature: None,
         error: error.into(),
         read_ts,
      }
   }
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
      let read_ts = proto.read_ts.ok_or_else(|| anyhow!("read_ts is None"))?;
      let id = proto.id.clone().ok_or_else(|| anyhow!("id is None"))?;
      let res = Self {
         id: id.try_into().with_context(|| anyhow!("Failed to convert proto id to id"))?,
         temperature: proto.temperature,
         error: proto.error,
         read_ts: proto_timestamp_to_chrono(read_ts)?.into(),
      };
      Ok(res)
   }
}


// ===========================================================================================================
// Logger


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
      let sensor_id = SensorId::new();
      let expected: Measurement = Measurement {
         id: MeasurementId::new(sensor_id),
         read_ts: MicroSecTs(ts),
         temperature: Some(26.8),
         error: "error1".to_string(),
      };
      let proto: crate::pb::Measurement = expected.clone().into();
      assert_eq!(
         proto,
         crate::pb::Measurement {
            id: Some(expected.id.clone().into()),
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
