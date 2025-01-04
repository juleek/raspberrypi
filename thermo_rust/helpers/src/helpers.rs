
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
   pub sensor:      String,
   pub temperature: Option<f64>,
   pub errors:      Vec<String>,
}

pub type Rx = tokio::sync::mpsc::Receiver<Measurement>;
pub type Tx = tokio::sync::mpsc::Sender<Measurement>;

impl From<Measurement> for agg_proto::Measurement {
   fn from(value: Measurement) -> Self {
      Self { sensor:      value.sensor,
             temperature: value.temperature,
             errors:      value.errors, }
   }
}

impl From<agg_proto::Measurement> for Measurement {
   fn from(value: agg_proto::Measurement) -> Self {
      Self { sensor:      value.sensor,
             temperature: value.temperature,
             errors:      value.errors, }
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
      let expected: Measurement = Measurement { sensor:      "ambient".to_string(),
                                                temperature: Some(26.8),
                                                errors:      vec!["error1".to_string(), "error2".to_string()], };
      let proto: agg_proto::Measurement = expected.clone().into();
      assert_eq!(proto, agg_proto::Measurement { sensor:      "ambient".to_string(),
                                                 temperature: Some(26.8),
                                                 errors:      vec!["error1".to_string(), "error2".to_string()], });

      let actual: Measurement = proto.into();
      assert_eq!(actual, expected);

      Ok(())
   }
}
