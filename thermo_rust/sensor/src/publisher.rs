use anyhow::{anyhow, Context, Result};

//
// ===========================================================================================================
// Measurements

#[derive(Debug, Clone, PartialEq)]
struct Measurements {
   age: chrono::Duration,
   by_id: std::collections::HashMap<common::MeasurementId, common::Measurement>,
   by_send_ts: std::collections::BTreeMap<chrono::DateTime<chrono::Utc>, Vec<common::MeasurementId>>,
}

impl Default for Measurements {
   fn default() -> Self {
      Self {
         age: chrono::Duration::minutes(2),
         by_id: Default::default(),
         by_send_ts: Default::default(),
      }
   }
}

impl Measurements {
   fn add(&mut self, m: common::Measurement, now: chrono::DateTime<chrono::Utc>) {
      let id = m.id.clone();
      self.by_id.insert(id.clone(), m);
      self.by_send_ts.entry(now).or_default().push(id);
   }
   fn remove(&mut self, id: &common::MeasurementId) {
      self.by_id.remove(id);

      let mut to_remove = Vec::new();
      for (ts, vec) in &mut self.by_send_ts {
         vec.retain(|x| x != id);
         if vec.is_empty() {
            to_remove.push(*ts);
         }
      }
      for ts in to_remove {
         self.by_send_ts.remove(&ts);
      }
   }

   fn get_next_to_retry(&mut self, now: chrono::DateTime<chrono::Utc>) -> Option<common::Measurement> {
      let deadline = now - self.age;
      let next_to_try = self.by_send_ts.range_mut(..=deadline).next_back();

      let Some((ts, vec)) = next_to_try else {
         return None;
      };
      let ts = ts.clone();
      let id = vec.pop().unwrap();

      if vec.is_empty() {
         self.by_send_ts.remove(&ts);
      }

      let measurement = self.by_id.get(&id).unwrap().clone();
      self.add(measurement.clone(), now);
      Some(measurement.clone())
   }
}




//
// ===========================================================================================================
// State

struct State {
   thread_rx: common::Rx,
   measurements: Measurements,
}
impl State {
   fn new(thread_rx: common::Rx) -> Self {
      Self {
         thread_rx,
         measurements: Default::default(),
      }
   }


   fn on_new_measurement(&mut self, measurement: common::Measurement) -> common::pb::StoreMeasurementReq {
      let req = common::pb::StoreMeasurementReq {
         measurement: Some(measurement.clone().into()),
      };
      self.measurements.add(measurement, chrono::Utc::now());
      if self.measurements.by_id.len() > 100 {
         log::warn!("Measurements size: {}", self.measurements.by_id.len());
      }
      req
   }
   fn pull_from_rx(&mut self) {
      while let Ok(measurement) = self.thread_rx.try_recv() {
         self.on_new_measurement(measurement);
      }
   }
   fn remove_confirmed(&mut self, id: common::MeasurementId) { self.measurements.remove(&id); }
}



async fn one_iteration(
   ct: &tokio_util::sync::CancellationToken,
   server_host_port: &str, // localhost:1234 (without scheme)
   state: &mut State,
   client_config_provider: &common::tls::ClientConfigProvider,
) -> Result<()> {
   let url = format!("https://{server_host_port}");
   let url = url::Url::parse(&url).with_context(|| anyhow!("Failed to parse: {url}"))?;
   let host = url.host().ok_or_else(|| anyhow!("There is no host in {url}"))?;
   state.pull_from_rx();
   let channel = tonic::transport::Endpoint::from_shared(url.to_string())
      .with_context(|| anyhow!("Failed to create channel for {server_host_port}"))?
      .tls_config(client_config_provider.create_for(&host.to_string()))
      .with_context(|| anyhow!("Failed to set tls config for {server_host_port}"))?
      .connect_timeout(std::time::Duration::from_secs(10))
      .connect()
      .await
      .with_context(|| anyhow!("Failed to connect to {server_host_port}"))?;
   let mut client = common::pb::agg_client::AggClient::new(channel);

   let (tx_outbound, rx_outbound) = tokio::sync::mpsc::channel(10);
   let outbound = tokio_stream::wrappers::ReceiverStream::new(rx_outbound);
   let mut inbound_stream = client.store_measurement(outbound).await?.into_inner();


   let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(10));
   loop {
      tokio::select! {
         _ = ct.cancelled() => {
            return Ok(());
         },
         Some(measurement) = state.thread_rx.recv() => {
            let req = state.on_new_measurement(measurement);
            log::info!("Sending: {req:?} to {server_host_port}");
            tx_outbound.send(req.clone()).await.with_context(|| anyhow!("Failed to send measurement {req:?}"))?;
         },
         confirmed = inbound_stream.message() => {
            match confirmed {
               Ok(Some(confirmed)) => {
                  let Some(confirmed) = confirmed.confirmed else {continue};
                  let Ok(confirmed) = confirmed.try_into() else {continue};
                  state.remove_confirmed(confirmed);
               },
               Ok(None) => { return Err(anyhow!("Received an empty response from the stream"));   },
               Err(e) => {   return Err(anyhow!("Got error from grpc: {e:?}"));  }
            }
         },
         _ = interval.tick() => {
            let Some(to_retry) = state.measurements.get_next_to_retry(chrono::Utc::now()) else {
               continue
            };
            let req = common::pb::StoreMeasurementReq {
               measurement: Some(to_retry.into()),
            };
            tx_outbound.send(req.clone()).await.with_context(|| anyhow!("Failed to send measurement {req:?}"))?;
         }
      }
   }
}



pub async fn poll_and_publish_forever(
   ct: &tokio_util::sync::CancellationToken,
   thread_rx: common::Rx,
   server_host_port: &str,
   client_config_provider: common::tls::ClientConfigProvider,
) -> Result<()> {
   let mut state = State::new(thread_rx);
   loop {
      let res = one_iteration(ct, server_host_port, &mut state, &client_config_provider).await;
      if let Err(e) = res {
         log::warn!("Failed to do one iteration: {e:?}");
      }
      if ct.is_cancelled() {
         return Ok(());
      }
      tokio::time::sleep(std::time::Duration::from_millis(290)).await;
   }
}



//
// ===========================================================================================================
// Tests


#[cfg(test)]
mod tests {
   use super::*;
   use pretty_assertions::assert_eq;

   fn ts_ymd(year: i32, month: u32, day: u32) -> common::MicroSecTs {
      use chrono::TimeZone;
      let ts = chrono::Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).earliest().unwrap();
      common::MicroSecTs(ts)
   }

   fn create_id(sensor_id: &common::SensorId, ticket: i64) -> common::MeasurementId {
      common::MeasurementId {
         sensor_id: sensor_id.clone(),
         index: ticket,
      }
   }

   fn measurement(id: &common::MeasurementId) -> common::Measurement {
      common::Measurement {
         id: id.clone(),
         temperature: Some(26.8),
         read_ts: ts_ymd(2024, 1, 1),
         error: "error1".to_string(),
      }
   }

   #[test]
   fn test_measurements_remove_removes_just_one_element_with_ts() {
      let now = chrono::Utc::now();
      let sensor_id = &common::SensorId::new();
      let id1 = create_id(sensor_id, 123);
      let id2 = create_id(sensor_id, 234);

      let mut measurements = Measurements::default();
      measurements.add(measurement(&id1), now);
      measurements.add(measurement(&id2), now);
      measurements.remove(&id1);

      let mut expected = Measurements::default();
      expected.add(measurement(&id2), now);

      assert_eq!(measurements, expected);
   }

   #[test]
   fn test_measurements_remove_not_remove_anything_if_element_not_found() {
      let now = chrono::Utc::now();
      let sensor_id = &common::SensorId::new();
      let id1 = create_id(sensor_id, 123);
      let id2 = create_id(sensor_id, 234);

      let mut measurements = Measurements::default();
      measurements.add(measurement(&id1), now);

      measurements.remove(&id2);

      let mut expected = Measurements::default();
      expected.add(measurement(&id1), now);

      assert_eq!(measurements, expected);
   }

   #[test]
   fn test_measurements_remove_if_no_elements() {
      let sensor_id = &common::SensorId::new();
      let id1 = create_id(&sensor_id, 123);

      let mut measurements = Measurements::default();

      measurements.remove(&id1);

      let expected = Measurements::default();
      assert_eq!(measurements, expected);
   }
}
