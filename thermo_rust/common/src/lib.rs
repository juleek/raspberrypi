pub mod pb;
use anyhow::{anyhow, Result};

fn proto_timestamp_to_chrono(proto: prost_types::Timestamp) -> Result<chrono::DateTime<chrono::Utc>> {
   chrono::DateTime::from_timestamp(proto.seconds, proto.nanos as u32)
      .map_or_else(|| Err(anyhow!("Failed to convert proto: {proto} to chrono")), Ok)
}

fn chrono_timestamp_to_proto(dt: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
   prost_types::Timestamp {
      seconds: dt.timestamp(),
      nanos: dt.timestamp_subsec_nanos() as i32,
   }
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Measurement {
   pub read_ts: chrono::DateTime<chrono::Utc>,
   pub sensor: String,
   pub temperature: Option<f64>,
   pub error: String,
}

pub type Rx = tokio::sync::mpsc::Receiver<Measurement>;
pub type Tx = tokio::sync::mpsc::Sender<Measurement>;

impl From<Measurement> for crate::pb::Measurement {
   fn from(value: Measurement) -> Self {
      Self {
         sensor: value.sensor,
         temperature: value.temperature,
         error: value.error,
         read_ts: Some(chrono_timestamp_to_proto(value.read_ts)),
      }
   }
}

impl TryFrom<crate::pb::Measurement> for Measurement {
   type Error = anyhow::Error;

   fn try_from(proto: crate::pb::Measurement) -> Result<Self, Self::Error> {
      let read_ts = proto.read_ts.ok_or_else(|| anyhow!("read_ts is None in {proto:?}"))?;
      let res = Self {
         sensor: proto.sensor,
         temperature: proto.temperature,
         error: proto.error,
         read_ts: proto_timestamp_to_chrono(read_ts)?,
      };
      Ok(res)
   }
}

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
         read_ts: ts,
         sensor: "ambient".to_string(),
         temperature: Some(26.8),
         error: "error1".to_string(),

      };
      let proto: crate::pb::Measurement = expected.clone().into();
      assert_eq!(
         proto,
         crate::pb::Measurement {
            read_ts: Some(chrono_timestamp_to_proto(ts)),
            sensor: "ambient".to_string(),
            temperature: Some(26.8),
            error: "error1".to_string(),
         }
      );

      let actual: Measurement = proto.into();
      assert_eq!(actual, expected);

      Ok(())
   }
}
